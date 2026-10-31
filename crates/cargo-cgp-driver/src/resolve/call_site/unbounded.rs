//! A consumer method called on a type parameter its generic item does not bound by the consumer.

use cargo_cgp_error_processing::{Causes, ChainNode, Leaf, Resolved, ResolvedClass};
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{self as hir, QPath};
use rustc_middle::ty::{self, TyCtxt, Upcast as _};

use crate::resolve::call_site::MethodCall;
use crate::resolve::label::{label_for, render_ty};
use crate::resolve::query::{
    CallTrait, declared_binding_ty, local_binding, peel_hir_refs, traits_with_method,
};

/// The cause of a consumer-method call whose receiver is a binding typed by a type parameter of
/// the enclosing item (`context.greet()` for `context: &Context` in `fn run<Context>`), when the
/// item does not bound the parameter by the one consumer trait in scope that declares the method.
/// The method exists only where the parameter has that bound, so the call fails for the bound
/// alone, whatever any context wires. The block is the consumer class `[CGP-E001]`, its cause a
/// [`Leaf::UnboundedParameter`] whose `help` names the bound to add.
///
/// `None` for any other receiver, when no consumer trait or more than one declares the method, when
/// the trait takes parameters besides `Self` (the bound to add would need arguments the call does
/// not show), or when the item already declares the bound.
pub(crate) fn unbounded_receiver<'tcx>(
    tcx: TyCtxt<'tcx>,
    call: &MethodCall<'tcx>,
) -> Option<Resolved> {
    let binding = local_binding(call.receiver)?;
    let declared = peel_hir_refs(declared_binding_ty(tcx, binding)?);
    let hir::TyKind::Path(QPath::Resolved(None, ty_path)) = declared.kind else {
        return None;
    };
    let Res::Def(DefKind::TyParam, param_did) = ty_path.res else {
        return None;
    };
    let item_did = tcx.parent(param_did);
    // The parameter's type is built from the item's generics: `type_of` is not defined on a type
    // parameter without a default, and the bug it reports would re-enter the diagnostic context.
    let index = tcx
        .generics_of(item_did)
        .param_def_id_to_index(tcx, param_did)?;
    let param_ty = ty::Ty::new_param(tcx, index, tcx.item_name(param_did));

    let in_scope = tcx.in_scope_traits(call.hir_id).unwrap_or_default();
    let mut consumers =
        traits_with_method(tcx, call.method)
            .into_iter()
            .filter(|&(trait_did, _, kind)| {
                kind == CallTrait::Consumer
                    && in_scope
                        .iter()
                        .any(|candidate| candidate.def_id == trait_did)
            });
    let (trait_did, _, _) = consumers.next()?;
    if consumers.next().is_some() || tcx.generics_of(trait_did).count() != 1 {
        return None;
    }
    let bounded = tcx
        .clauses_of(item_did)
        .clauses
        .iter()
        .filter_map(|&(clause, _)| clause.as_trait_clause())
        .any(|bound| bound.def_id() == trait_did && bound.skip_binder().self_ty() == param_ty);
    if bounded {
        return None;
    }

    let trait_ref = ty::TraitRef::new(tcx, trait_did, [param_ty]);
    let leaf = Leaf::UnboundedParameter {
        param: render_ty(tcx, param_ty),
        trait_name: tcx.item_name(trait_did).to_string(),
        item: tcx.item_name(item_did).to_string(),
    };
    let mut chain = Vec::new();
    if let Some(hop) = label_for(tcx, ty::Binder::dummy(trait_ref).upcast(tcx), param_ty) {
        chain.push(ChainNode::Hop(hop));
    }
    chain.push(ChainNode::Leaf(leaf.clone()));
    Some(Resolved {
        context: render_ty(tcx, param_ty),
        consumers: vec![tcx.item_name(trait_did).to_string()],
        class: ResolvedClass::Consumer,
        // A type parameter carries no wiring, so it is named plainly.
        subject_is_context: false,
        causes: Causes::from_sub_chains([(leaf, chain)]),
    })
}
