//! The forwarding-entry anchor: a `Key -> Table` wiring entry whose table lacks the key.

use cargo_cgp_error_processing::Resolved;
use rustc_middle::ty::{TyCtxt, TypeVisitableExt as _};
use rustc_span::Span;

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::anchor::consumer_obligation;
use crate::resolve::cache::ResolveCache;
use crate::resolve::cgp_item::{component_params, find_cgp_trait, is_cgp_item, marker_to_consumer};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve an error the compiler raises on a `Key -> Table` entry of `delegate_components!`. The
/// entry lowers to `impl DelegateComponent<Key> for Context where Table: DelegateComponent<Key>`,
/// with the bound spanned onto the entry's `->`. When the table lacks the key, the compiler reports
/// the unprovable bound under the entry's `-> Table` caret (and the `Delegate` projection it leaves
/// unsized), before any check or call that uses the entry. The entry wires `Key` on the context, so
/// its failure is the consumer obligation that wiring serves, seeded and walked as a check of the
/// component would be; the cause and the chain are then the check's, and a later report of the
/// same failure de-duplicates against this one.
///
/// `None` when no local entry's forwarding bound sits at the caret, when the key is not a bare
/// component marker without parameters (its consumer obligation needs values the entry does not
/// name), or when the consumer obligation holds or walks to no cause.
pub fn resolve_forwarding_entry(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    primary: &[Span],
) -> Option<Resolved> {
    let delegate_did = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;
    for impl_did in tcx.all_impls(delegate_did).filter(|did| did.is_local()) {
        let impl_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        let context = impl_ref.self_ty();
        let key = impl_ref.args.type_at(1);
        let forwards_at_caret = tcx
            .clauses_of(impl_did)
            .clauses
            .iter()
            .any(|&(clause, span)| {
                let Some(bound) = clause.as_trait_clause() else {
                    return false;
                };
                let bound = bound.skip_binder().trait_ref;
                is_cgp_item(
                    tcx,
                    bound.def_id,
                    DELEGATE_COMPONENT_TRAIT,
                    CGP_COMPONENT_CRATE,
                ) && bound.args.type_at(1) == key
                    && bound.self_ty() != context
                    && primary
                        .iter()
                        .any(|caret| caret.lo() <= span.lo() && span.hi() <= caret.hi())
            });
        if !forwards_at_caret {
            continue;
        }
        if context.has_param() || key.has_param() || !component_params(tcx, key).is_empty() {
            return None;
        }
        let (consumer_did, _) = marker_to_consumer(tcx, key)?;
        let top = consumer_obligation(tcx, context, consumer_did, tcx.types.unit)?;
        if holds(tcx, top) {
            return None;
        }
        return resolve_leaves(tcx, cache, top);
    }
    None
}
