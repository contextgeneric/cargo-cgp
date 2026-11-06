//! The human-readable label each dependency-path predicate renders as.

use cargo_cgp_error_processing::DepNode;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT, IS_PROVIDER_FOR_TRAIT,
    REDIRECT_LOOKUP_TYPE,
};
use crate::resolve::data::{is_data_list_recursion, read_partial};
use crate::resolve::label::render_ty;
use crate::resolve::query::{
    carries_wiring, is_cgp_item, is_consumer_trait, is_namespace_lookup_trait, is_provider_trait,
};

/// The human-readable label for one predicate in a dependency path, replacing each CGP wiring
/// trait with the concept it stands for. Crucially it reads the **real** consumer and provider
/// trait obligations that the walk descends (`Ctx: ConsumerTrait<…>` and `Provider:
/// ProviderTrait<Ctx, …>`) and takes every name straight off the trait `DefId` and the
/// obligation's own type arguments. It does *not* read `CanUseComponent`/`IsProviderFor`: those
/// are the check-trait scaffolding cargo-cgp treats as removable, and the resolver never depends on
/// them for a name.
///
/// - A consumer-trait obligation on the context becomes the consumer-trait impl.
/// - A provider-trait obligation whose `Self` is a real provider becomes the provider-trait impl
///   (its trait, context, provider struct, and the component's extra parameters). A
///   `RedirectLookup<Table, Path>` provider is a namespace/`open` redirection hop instead, so a
///   chain of them reads as its successive hops, each naming the table it looks the path up in.
/// - A `HasField` obligation never reaches here: the walk classifies it as a root-cause leaf first.
/// - Any other trait (a user's own blanket trait or getter) is shown as a trait impl for its self.
///
/// The steps that carry no information for a reader return `None` and are dropped: the
/// `CanUseComponent`/`IsProviderFor` scaffolding, a provider-trait obligation *for the context
/// itself* (delegation routing), the `DelegateComponent` table lookup, a namespace lookup, and an
/// extensible-data trait's recursion over a type-level list. A trait implemented for a builder or
/// an extractor names that partial value in words. The missing-delegate leaf a wiring chain bottoms
/// out on is not rendered here: the caller
/// ([`resolve_leaves`](crate::resolve::walk::resolve_leaves)) re-states the root cause as the
/// terminal leaf.
pub fn label_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
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
        || is_data_list_recursion(tcx, pred)
    {
        return None;
    }

    // Note: a `HasField` obligation is never labeled here (`resolve_node` classifies it as a
    // terminal root-cause leaf before `label_for` is reached), so there is no interior HasField
    // hop.

    if is_provider_trait(tcx, did) {
        // A provider-trait obligation for the context itself is delegation routing, dropped.
        if self_ty == context {
            return None;
        }
        if let Some(node) = provider_label(tcx, trait_ref) {
            return Some(node);
        }
    }

    if is_consumer_trait(tcx, did) && self_ty == context {
        // `Ctx: ConsumerTrait<Params…>`: the consumer name and its parameters read directly.
        let generics = trait_generics(tcx, trait_ref, 1);
        return Some(DepNode::Consumer {
            trait_ref: format!("{}{generics}", tcx.item_name(did)),
            context: render_ty(tcx, self_ty),
            is_context: carries_wiring(tcx, self_ty),
        });
    }

    // A user's own blanket or getter trait, or a terminal ordinary bound, with its arguments. A
    // builder or an extractor is named in words, since its type is a derive's generated partial
    // value.
    let trait_ref = format!(
        "{}{}",
        tcx.item_name(did),
        trait_generics(tcx, trait_ref, 1)
    );
    if let Some(partial) = read_partial(tcx, self_ty) {
        return Some(DepNode::PartialTrait {
            trait_ref,
            value: partial.value(),
        });
    }
    Some(DepNode::Trait {
        trait_ref,
        self_ty: render_ty(tcx, self_ty),
    })
}

/// The node of a provider-trait obligation not for the context, or `None` for a trait whose shape
/// `#[cgp_component]` does not emit.
fn provider_label<'tcx>(tcx: TyCtxt<'tcx>, trait_ref: ty::TraitRef<'tcx>) -> Option<DepNode> {
    let self_ty = trait_ref.self_ty();
    // A `RedirectLookup<Table, Path>` provider is a namespace/`open` redirection hop, not a
    // real provider impl, so a chain of them reads as its successive hops. `Table` is where the
    // path is looked up: the context for its own `open` or `namespace`, but an aggregate
    // provider for a bundle that opens its own components, so the hop names it rather than the
    // context. The dispatched key (the provider trait's own parameters, skipping `Self` = the
    // lookup and the context) is carried as the node's identity so two lookups along the same
    // route for different keys stay distinct; it is not rendered, since the key already shows
    // on the child provider node.
    if let Some((table, path)) = redirect_lookup_args(tcx, self_ty) {
        return Some(DepNode::Redirect {
            path: path.to_string(),
            table: table.to_string(),
            key: trait_generics(tcx, trait_ref, 2),
        });
    }
    // `Provider: ProviderTrait<Ctx, Params…>`: trait name, context, provider, and the
    // component's extra parameters all read straight off the obligation, no marker or map. The
    // context is the first type argument after `Self`, indexed by *type* position because a
    // component's lifetime parameters sort ahead of the context in the argument list
    // (`ReferenceGetter<'a, Ctx, T>`); indexing by argument position would land on the region
    // and abort the compiler. A trait with no such argument is not a shape `#[cgp_component]`
    // emits, so it falls through to the plain label.
    if let Some(provider_context) = trait_ref.args.types().nth(1) {
        let generics = trait_generics(tcx, trait_ref, 2);
        return Some(DepNode::Provider {
            trait_ref: format!("{}{generics}", tcx.item_name(trait_ref.def_id)),
            context: render_ty(tcx, provider_context),
            provider: render_ty(tcx, self_ty),
        });
    }
    None
}

/// Render a trait obligation's arguments as a generic list (`<u32, u64>`, `<'_, str>`), or the
/// empty string when there are none, dropping the first `skip` *type* arguments. For a consumer
/// obligation `Ctx: C<A, B>` that is the `Self`=`Ctx` (skip 1), and for a provider obligation
/// `P: T<Ctx, A, B>` the `Self` and the leading context (skip 2), so the component's own parameters
/// reattach to the trait exactly as written, read straight off the obligation rather than from a
/// `CanUseComponent`/`IsProviderFor` params tuple. Lifetime arguments are kept in place: the walk
/// erases regions, so an unnamed one reads `'_`, as it does in a rendered type (`App<'_>`).
pub fn trait_generics<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_ref: ty::TraitRef<'tcx>,
    skip: usize,
) -> String {
    let mut types = 0;
    let params: Vec<String> = trait_ref
        .args
        .iter()
        .filter_map(|arg| {
            if let Some(ty) = arg.as_type() {
                types += 1;
                (types > skip).then(|| render_ty(tcx, ty))
            } else if let Some(region) = arg.as_region() {
                Some(match region.get_name(tcx) {
                    Some(name) => name.to_string(),
                    None => "'_".to_owned(),
                })
            } else {
                Some(arg.to_string())
            }
        })
        .collect();
    if params.is_empty() {
        String::new()
    } else {
        format!("<{}>", params.join(", "))
    }
}

/// The table and path of a `RedirectLookup<Table, Path>` provider, or `None` when `provider` is not
/// a `RedirectLookup`. Anchored to the CGP crate that defines the type, so a same-named type
/// elsewhere is never mistaken for it.
pub fn redirect_lookup_args<'tcx>(
    tcx: TyCtxt<'tcx>,
    provider: Ty<'tcx>,
) -> Option<(Ty<'tcx>, Ty<'tcx>)> {
    let ty::Adt(def, args) = provider.kind() else {
        return None;
    };
    is_cgp_item(tcx, def.did(), REDIRECT_LOOKUP_TYPE, CGP_COMPONENT_CRATE)
        .then(|| (args.type_at(0), args.type_at(1)))
}
