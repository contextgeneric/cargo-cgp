//! Reading the context type off the call's receiver expression.

use rustc_hir::def::{CtorKind, CtorOf, DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, QPath};
use rustc_middle::ty::{Ty, TyCtxt};

use crate::resolve::call_site::{call_output_ty, item_ty, lower_hir_ty, peel_hir_refs};

/// The context type a call's receiver expression names — the *receiver* is what carries the
/// context in a consumer-method call, so this is the anchor's one source of it. The type is read
/// syntactically: a path to a binding follows the binding (a `let` typed by its annotation or its
/// initializer, a fn parameter typed by the enclosing signature); a struct literal, unit-struct
/// value, const, or static names its type directly; a plain constructor call (`MyApp::new()`)
/// takes the callee's declared return type; references are peeled on the way. `None` for a
/// receiver whose type only typeck could know (a method call's result, a field access).
pub(crate) fn receiver_context<'tcx>(tcx: TyCtxt<'tcx>, expr: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    match expr.kind {
        ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
            Res::Local(binding) => local_binding_context(tcx, binding),
            // A unit-struct value used directly (`MyCliApp.handle(…)`) resolves to the struct's
            // const constructor.
            Res::Def(DefKind::Ctor(CtorOf::Struct, CtorKind::Const), ctor_did) => {
                item_ty(tcx, tcx.parent(ctor_did))
            }
            Res::Def(DefKind::Const { .. } | DefKind::Static { .. }, did) => item_ty(tcx, did),
            _ => None,
        },
        ExprKind::Struct(qpath, ..) => {
            let QPath::Resolved(None, path) = qpath else {
                return None;
            };
            let Res::Def(DefKind::Struct, did) = path.res else {
                return None;
            };
            item_ty(tcx, did)
        }
        // A plain constructor call: the callee's declared return type is the receiver's type,
        // read from its (collection-cached) signature — still no typeck results.
        ExprKind::Call(callee, _) => Some(call_output_ty(tcx, callee)?.peel_refs()),
        ExprKind::AddrOf(_, _, inner) | ExprKind::Unary(hir::UnOp::Deref, inner) => {
            receiver_context(tcx, inner)
        }
        _ => None,
    }
}

/// The type of the binding a receiver path resolves to. A `let` with a type annotation supplies
/// it directly; a `let` without one is typed by its initializer expression (a struct literal,
/// usually); a fn parameter is typed by the matching input of the enclosing signature. All three
/// are syntactic — no typeck results are consulted.
fn local_binding_context<'tcx>(tcx: TyCtxt<'tcx>, binding: hir::HirId) -> Option<Ty<'tcx>> {
    match tcx.parent_hir_node(binding) {
        hir::Node::LetStmt(let_stmt) => {
            if let Some(ty) = let_stmt.ty {
                return lower_hir_ty(tcx, peel_hir_refs(ty));
            }
            receiver_context(tcx, let_stmt.init?)
        }
        hir::Node::Param(param) => {
            let owner = tcx.hir_enclosing_body_owner(binding);
            let body = tcx.hir_body_owned_by(owner);
            let index = body
                .params
                .iter()
                .position(|candidate| candidate.pat.hir_id == param.pat.hir_id)?;
            let decl = tcx.hir_node_by_def_id(owner).fn_decl()?;
            lower_hir_ty(tcx, peel_hir_refs(decl.inputs.get(index)?))
        }
        _ => None,
    }
}
