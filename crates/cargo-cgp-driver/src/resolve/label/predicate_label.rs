//! The human-readable label each dependency-path predicate renders as.

use cargo_cgp_error_processing::DepNode;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT,
    REDIRECT_LOOKUP_TYPE,
};
use crate::resolve::cgp_item::{
    is_cgp_item, is_consumer_trait, is_namespace_lookup_trait, is_provider_trait,
};
use crate::resolve::label::render_ty;

/// The human-readable label for one predicate in a dependency path, replacing each CGP wiring
/// trait with the concept it stands for. Crucially it reads the **real** consumer and provider
/// trait obligations that the walk descends — `Ctx: ConsumerTrait<…>` and `Provider:
/// ProviderTrait<Ctx, …>` — and takes every name straight off the trait `DefId` and the
/// obligation's own type arguments. It does *not* read `CanUseComponent`/`IsProviderFor`: those
/// are the check-trait scaffolding cargo-cgp treats as removable, and the resolver never depends on
/// them for a name.
///
/// - A consumer-trait obligation on the context becomes the consumer-trait impl.
/// - A provider-trait obligation whose `Self` is a real provider becomes the provider-trait impl
///   (its trait, context, provider struct, and the component's extra parameters). A
///   `RedirectLookup<Ctx, Path>` provider is a namespace/`open` redirection hop instead, so a chain
///   of them reads as its successive hops.
/// - `HasField` becomes the field-trait impl (the field and the struct that must carry it).
/// - Any other trait — a user's own capability or getter — is shown as a trait impl for its self.
///
/// The steps that carry no information for a reader return `None` and are dropped: the
/// `CanUseComponent`/`IsProviderFor` scaffolding, a provider-trait obligation *for the context
/// itself* (delegation routing), the `DelegateComponent` table lookup, and a namespace lookup. The
/// missing-delegate leaf a wiring chain bottoms out on is not rendered here — the caller
/// ([`resolve_leaves`](crate::resolve::walk::resolve_leaves)) re-states the root cause as the
/// terminal leaf.
pub(crate) fn label_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
) -> Option<DepNode> {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;
    let self_ty = trait_ref.self_ty();

    // The check-trait scaffolding and table plumbing carry nothing for the reader; the real
    // consumer/provider obligations beside them do. Dropping these is also what keeps the resolver
    // off `IsProviderFor`: it is never read for a name, only recognized here to be discarded.
    if is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE)
        || is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_namespace_lookup_trait(tcx, did)
    {
        return None;
    }

    // Note: a `HasField` obligation is never labeled here — `resolve_node` classifies it as a
    // terminal root-cause leaf before `label_for` is reached — so there is no interior HasField hop.

    if is_provider_trait(tcx, did) {
        // A provider-trait obligation for the context itself is delegation routing, dropped.
        if self_ty == context {
            return None;
        }
        // A `RedirectLookup<Ctx, Path>` provider is a namespace/`open` redirection hop, not a real
        // provider impl, so a chain of them reads as its successive hops. The dispatched key (the
        // provider trait's own parameters, skipping `Self` = the lookup and the context) is carried
        // as the node's identity so two lookups along the same route for different keys stay
        // distinct — it is not rendered, since the key already shows on the child provider node.
        if let Some(path) = redirect_path(tcx, self_ty) {
            return Some(DepNode::Redirect {
                path: path.to_string(),
                context: context.to_string(),
                key: trait_generics(tcx, trait_ref, 2),
            });
        }
        // `Provider: ProviderTrait<Ctx, Params…>` — trait name, context, provider, and the
        // component's extra parameters all read straight off the obligation, no marker or map. The
        // context is the first type argument after `Self`, indexed by *type* position because a
        // component's lifetime parameters sort ahead of the context in the argument list
        // (`ReferenceGetter<'a, Ctx, T>`); indexing by argument position would land on the region
        // and abort the compiler. A trait with no such argument is not a shape `#[cgp_component]`
        // emits, so it falls through to the plain label.
        if let Some(provider_context) = trait_ref.args.types().nth(1) {
            let generics = trait_generics(tcx, trait_ref, 2);
            return Some(DepNode::Provider {
                trait_ref: format!("{}{generics}", tcx.item_name(did)),
                context: render_ty(tcx, provider_context),
                provider: render_ty(tcx, self_ty),
            });
        }
    }

    if is_consumer_trait(tcx, did) && self_ty == context {
        // `Ctx: ConsumerTrait<Params…>` — the consumer name and its parameters read directly.
        let generics = trait_generics(tcx, trait_ref, 1);
        return Some(DepNode::Consumer {
            trait_ref: format!("{}{generics}", tcx.item_name(did)),
            context: render_ty(tcx, self_ty),
        });
    }

    // A user's own capability/getter trait, or a terminal ordinary bound.
    Some(DepNode::Trait {
        trait_ref: tcx.item_name(did).to_string(),
        self_ty: render_ty(tcx, self_ty),
    })
}

/// Render a trait obligation's type arguments after `skip` leading ones as a generic list —
/// `<u32, u64>` — or the empty string when there are none. For a consumer obligation `Ctx:
/// C<A, B>` the arguments are spread (skip the `Self`=`Ctx`), and for a provider obligation `P:
/// T<Ctx, A, B>` likewise (skip `Self` and the leading context), so the component's extra
/// parameters reattach to the trait exactly as written — read straight off the obligation rather
/// than from a `CanUseComponent`/`IsProviderFor` params tuple.
pub(crate) fn trait_generics<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_ref: ty::TraitRef<'tcx>,
    skip: usize,
) -> String {
    let params: Vec<String> = trait_ref
        .args
        .types()
        .skip(skip)
        .map(|ty| render_ty(tcx, ty))
        .collect();
    if params.is_empty() {
        String::new()
    } else {
        format!("<{}>", params.join(", "))
    }
}

/// The redirect path of a `RedirectLookup<Ctx, Path>` provider — its second type argument — or
/// `None` when `provider` is not a `RedirectLookup`. Anchored to the CGP crate that defines the
/// type, so a same-named type elsewhere is never mistaken for it.
fn redirect_path<'tcx>(tcx: TyCtxt<'tcx>, provider: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let ty::Adt(def, args) = provider.kind() else {
        return None;
    };
    is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE).then(|| args.type_at(1))
}
