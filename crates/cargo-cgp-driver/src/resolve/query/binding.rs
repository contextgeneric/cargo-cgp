//! Reading a local binding off an expression, and the type it is declared with, in the two forms
//! the code uses: the HIR type as written, and the input of the enclosing function's signature.

use rustc_hir::def::{DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, HirId, Node, QPath};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::LocalDefId;

/// The local binding `expr` names, when it is a plain path to one.
pub fn local_binding(expr: &Expr<'_>) -> Option<HirId> {
    let ExprKind::Path(QPath::Resolved(None, path)) = expr.kind else {
        return None;
    };
    let Res::Local(binding) = path.res else {
        return None;
    };
    Some(binding)
}

/// The type a binding is declared with, references kept: a fn parameter's input type, or a `let`'s
/// annotation. `None` for any other binding. An argument passed on by reference (`greet_all(app)`
/// for `app: &App`) needs the reference to line up with the declared input it fills.
pub fn declared_binding_ty<'tcx>(tcx: TyCtxt<'tcx>, binding: HirId) -> Option<&'tcx hir::Ty<'tcx>> {
    match tcx.parent_hir_node(binding) {
        Node::LetStmt(let_stmt) => let_stmt.ty,
        Node::Param(param) => {
            let owner = tcx.hir_enclosing_body_owner(binding);
            let index = param_index(tcx, owner, param)?;
            let decl = tcx.hir_node_by_def_id(owner).fn_decl()?;
            decl.inputs.get(index)
        }
        _ => None,
    }
}

/// The declared type of the fn parameter `binding` names, with its late-bound lifetimes left bound
/// ([`param_declared_input_binder`]). A `#[cgp_impl]` provider's renamed `self`, declared
/// `&__Context__`, is the case this was written for.
pub fn param_declared_input<'tcx>(tcx: TyCtxt<'tcx>, binding: HirId) -> Option<Ty<'tcx>> {
    param_declared_input_binder(tcx, binding).map(ty::Binder::skip_binder)
}

/// The declared type of the fn parameter `binding` names, under the signature's binder of
/// late-bound lifetimes, read from the enclosing function's signature rather than from typeck
/// results, which do not exist yet for the body whose type checking is reporting the error. `None`
/// when `binding` is not a parameter.
///
/// An `async fn` body sees its parameters through fresh bindings, since lowering moves each into the
/// future with an untyped `let __context__ = __context__;`, so such a forwarding `let` is followed
/// back to the parameter it copies.
pub fn param_declared_input_binder<'tcx>(
    tcx: TyCtxt<'tcx>,
    mut binding: HirId,
) -> Option<ty::Binder<'tcx, Ty<'tcx>>> {
    let param = loop {
        match tcx.parent_hir_node(binding) {
            Node::Param(param) => break param,
            Node::LetStmt(let_stmt) if let_stmt.ty.is_none() => {
                let source = local_binding(let_stmt.init?)?;
                if source == binding {
                    return None;
                }
                binding = source;
            }
            _ => return None,
        }
    };
    let owner = tcx.hir_enclosing_body_owner(binding);
    if !matches!(tcx.def_kind(owner), DefKind::AssocFn | DefKind::Fn) {
        return None;
    }
    let index = param_index(tcx, owner, param)?;
    let sig = tcx
        .fn_sig(owner.to_def_id())
        .instantiate_identity()
        .skip_norm_wip();
    let input = *sig.skip_binder().inputs().get(index)?;
    Some(sig.rebind(input))
}

/// The position of `param` among the parameters of the body `owner` owns.
pub fn param_index(tcx: TyCtxt<'_>, owner: LocalDefId, param: &hir::Param<'_>) -> Option<usize> {
    tcx.hir_body_owned_by(owner)
        .params
        .iter()
        .position(|candidate| candidate.pat.hir_id == param.pat.hir_id)
}
