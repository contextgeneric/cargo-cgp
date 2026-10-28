//! Typing a value an extensible-data operation produces, through the chain of calls that built it.

use rustc_hir::def::{CtorOf, DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, PatKind, QPath};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode, Unnormalized};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Symbol};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::call_site::receiver_context;
use crate::resolve::data::is_data_operation;
use crate::resolve::query::{expr_written_ty, lower_hir_ty, peel_hir_refs};

/// The type of `expr` when it is a value one of CGP's extensible-data operations produces, or a
/// value the call-site anchor already types ([`receiver_context`]): a call to a data operation on
/// such a value (`Person::builder().build_field(…)`), a type-relative call to one
/// (`Person::builder()`), or a binding that a `let` or a tuple-variant pattern takes out of such a
/// value (`Err(remainder)` from a `downcast`). Each step reads the operation's declared signature
/// and normalizes its output in a fresh inference context, so no typeck results are needed. `None`
/// for any other expression, or a step whose output does not reduce to a concrete type.
pub(crate) fn data_expr_ty<'tcx>(tcx: TyCtxt<'tcx>, expr: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    match expr.kind {
        ExprKind::MethodCall(segment, receiver, args, _) => {
            let self_ty = data_expr_ty(tcx, receiver)?;
            data_call_output(tcx, self_ty, segment.ident.name, args, true)
        }
        ExprKind::Call(callee, args) => match callee.kind {
            ExprKind::Path(QPath::TypeRelative(qself, segment)) => {
                let self_ty = lower_hir_ty(tcx, peel_hir_refs(qself))?;
                data_call_output(tcx, self_ty, segment.ident.name, args, false)
            }
            _ => receiver_context(tcx, expr),
        },
        ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
            Res::Local(binding) => {
                pattern_binding_ty(tcx, binding).or_else(|| receiver_context(tcx, expr))
            }
            _ => receiver_context(tcx, expr),
        },
        _ => receiver_context(tcx, expr),
    }
}

/// The traits among CGP's extensible-data operations ([`is_data_operation`]) that declare an
/// associated fn named `name`, each with that fn's `DefId`. A method call (`receiver.name(…)`)
/// looks only at fns taking `self`.
pub(crate) fn data_fns_named(
    tcx: TyCtxt<'_>,
    name: Symbol,
    method_call: bool,
) -> Vec<(DefId, DefId)> {
    tcx.all_traits_including_private()
        .filter(|&trait_did| is_data_operation(tcx, trait_did))
        .filter_map(|trait_did| {
            let fn_did = tcx
                .associated_items(trait_did)
                .filter_by_name_unhygienic(name)
                .find(|item| match item.kind {
                    ty::AssocKind::Fn { has_self, .. } => has_self || !method_call,
                    _ => false,
                })?
                .def_id;
            Some((trait_did, fn_did))
        })
        .collect()
}

/// The output type of a call to the data operation named `name` on `self_ty`, through the first
/// data trait that declares it and that `self_ty` implements for the call's written arguments.
fn data_call_output<'tcx>(
    tcx: TyCtxt<'tcx>,
    self_ty: Ty<'tcx>,
    name: Symbol,
    args: &[Expr<'tcx>],
    method_call: bool,
) -> Option<Ty<'tcx>> {
    data_fns_named(tcx, name, method_call)
        .into_iter()
        .find_map(|(trait_did, fn_did)| {
            call_output(tcx, self_ty, trait_did, fn_did, args, method_call)
        })
}

/// The output type of calling `fn_did`, a fn of `trait_did`, with `self_ty` as `Self` and `args` as
/// written. Each argument whose type the call writes, or that is itself a data value, is unified
/// with its declared input, then the trait obligation is solved and the output normalized. `None`
/// when the obligation fails, or the output keeps an unknown or does not reduce.
fn call_output<'tcx>(
    tcx: TyCtxt<'tcx>,
    self_ty: Ty<'tcx>,
    trait_did: DefId,
    fn_did: DefId,
    args: &[Expr<'tcx>],
    method_call: bool,
) -> Option<Ty<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let cause = ObligationCause::dummy();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);

    let fn_args = infcx.fresh_args_for_item(DUMMY_SP, fn_did);
    ocx.eq(&cause, param_env, fn_args.type_at(0), self_ty)
        .ok()?;
    let sig = tcx.fn_sig(fn_did).instantiate(tcx, fn_args).skip_norm_wip();
    let sig = infcx.enter_forall_and_leak_universe(sig);
    // A method call's arguments follow its `self` input; a path call's arguments are every input.
    let skip = usize::from(method_call);
    for (arg, declared) in std::iter::zip(args, sig.inputs().iter().skip(skip)) {
        let written = expr_written_ty(&infcx, arg).or_else(|| data_expr_ty(tcx, arg));
        if let Some(written) = written {
            let _ = ocx.eq(&cause, param_env, *declared, written);
        }
    }

    let trait_ref = ty::TraitRef::from_assoc(tcx, trait_did, fn_args);
    ocx.register_obligation(Obligation::new(tcx, cause.clone(), param_env, trait_ref));
    let output = ocx.normalize(&cause, param_env, Unnormalized::new_wip(sig.output()));
    if ocx.try_evaluate_obligations().has_errors() {
        return None;
    }
    let output = tcx.erase_and_anonymize_regions(infcx.deeply_resolve_ignoring_regions(output));
    if output.has_non_region_infer()
        || output.has_placeholders()
        || matches!(output.kind(), ty::Alias(..))
    {
        return None;
    }
    Some(output)
}

/// The type of a binding a pattern takes out of a data value: a `let` (typed by its annotation or
/// its initializer), or a field of a tuple-struct or tuple-variant pattern (`Err(remainder)`) in a
/// `let`, an `if let`, or a `match` arm, whose scrutinee [`data_expr_ty`] types. `None` for any
/// other pattern shape.
fn pattern_binding_ty<'tcx>(tcx: TyCtxt<'tcx>, binding: hir::HirId) -> Option<Ty<'tcx>> {
    // Walk up from the binding to the pattern's root, recording each tuple-struct field it sits in.
    let mut fields: Vec<(DefId, usize)> = Vec::new();
    let mut current = binding;
    let root_ty = loop {
        match tcx.parent_hir_node(current) {
            hir::Node::Pat(parent) => {
                let PatKind::TupleStruct(QPath::Resolved(None, path), pats, dotdot) = parent.kind
                else {
                    return None;
                };
                if dotdot.as_opt_usize().is_some() {
                    return None;
                }
                let Res::Def(DefKind::Ctor(_, _), ctor_did) = path.res else {
                    return None;
                };
                let index = pats.iter().position(|pat| pat.hir_id == current)?;
                fields.push((ctor_did, index));
                current = parent.hir_id;
            }
            hir::Node::LetStmt(let_stmt) => {
                break match let_stmt.ty {
                    Some(ty) => lower_hir_ty(tcx, ty)?,
                    None => data_expr_ty(tcx, let_stmt.init?)?,
                };
            }
            hir::Node::Expr(Expr {
                kind: ExprKind::Let(let_expr),
                ..
            }) => break data_expr_ty(tcx, let_expr.init)?,
            hir::Node::Arm(_) => {
                let hir::Node::Expr(Expr {
                    kind: ExprKind::Match(scrutinee, ..),
                    ..
                }) = tcx.parent_hir_node(tcx.parent_hir_id(current))
                else {
                    return None;
                };
                break data_expr_ty(tcx, scrutinee)?;
            }
            _ => return None,
        }
    };
    // A plain binding (no tuple-struct around it) only has a type of its own for a `let`.
    if fields.is_empty() && !matches!(tcx.parent_hir_node(binding), hir::Node::LetStmt(_)) {
        return None;
    }
    fields
        .into_iter()
        .rev()
        .try_fold(root_ty, |ty, (ctor_did, index)| {
            let ty::Adt(adt, args) = ty.kind() else {
                return None;
            };
            let variant_owner = match tcx.def_kind(ctor_did) {
                DefKind::Ctor(CtorOf::Variant, _) => tcx.parent(tcx.parent(ctor_did)),
                _ => tcx.parent(ctor_did),
            };
            if variant_owner != adt.did() {
                return None;
            }
            let field = adt
                .variant_with_ctor_id(ctor_did)
                .fields
                .iter()
                .nth(index)?;
            Some(tcx.erase_and_anonymize_regions(field.ty(tcx, args).skip_norm_wip()))
        })
}
