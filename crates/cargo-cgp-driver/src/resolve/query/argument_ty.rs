//! Reading an argument's type through the enclosing function's signature.

use rustc_hir::{Expr, Node};
use rustc_infer::infer::{BoundRegionConversionTime, InferCtxt};
use rustc_middle::ty::Ty;
use rustc_span::DUMMY_SP;

use crate::resolve::query::{
    expr_written_ty, local_binding, lower_hir_ty, param_declared_input_binder,
};

/// The type an argument expression has without typeck: the type it writes ([`expr_written_ty`]),
/// or, for a path to a local binding, the type the binding is declared with (a fn parameter, or a
/// `let` with an annotation).
pub fn argument_ty<'tcx>(infcx: &InferCtxt<'tcx>, arg: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    if let Some(written) = expr_written_ty(infcx, arg) {
        return Some(written);
    }
    let binding = local_binding(arg)?;
    let tcx = infcx.tcx;
    if let Node::LetStmt(let_stmt) = tcx.parent_hir_node(binding)
        && let Some(annotation) = let_stmt.ty
    {
        return lower_hir_ty(tcx, annotation);
    }
    // A parameter's late-bound lifetimes (`shape: &Shape`) become fresh region variables, as they
    // would at a call, so the type unifies with a declared input.
    param_declared_input_binder(tcx, binding).map(|declared| {
        infcx.instantiate_binder_with_fresh_vars(
            DUMMY_SP,
            BoundRegionConversionTime::FnCall,
            declared,
        )
    })
}
