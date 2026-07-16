//! Rendering a dependency path's predicates as human-readable tree labels.
//!
//! This is where every CGP wiring trait is replaced by the concept it stands for, so the reader
//! never meets a raw `IsProviderFor` or `Symbol`. The rendered labels fold into a
//! [`DependencyTree`] spine that the [wording](cargo_cgp_error_processing::diagnosis) renders as
//! `cargo tree`-style text.

use cargo_cgp_error_processing::ComponentTraitNames;
use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::tree::DependencyTree;
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, DELEGATE_COMPONENT_TRAIT,
    HAS_FIELD_TRAIT, IS_PROVIDER_FOR_TRAIT,
};
use crate::resolve::cgp_item::{
    decode_symbol, is_cgp_item, is_namespace_lookup_trait, is_provider_trait,
};

/// The human-readable label for one predicate in a dependency path, replacing each CGP wiring
/// trait with the concept it stands for: `CanUseComponent` with the consumer-trait impl,
/// `IsProviderFor` with the provider-trait impl (its provider trait, context, and provider struct),
/// `HasField` with the field-trait impl (the field and the struct that must carry it). Any other
/// trait — a user's own consumer or getter capability — is shown as a trait impl for its self type.
///
/// The steps that carry no information for a reader are dropped so the chain stays legible: the
/// `DelegateComponent` wiring table, an `IsProviderFor` for the *context itself* (the delegation
/// routing, as opposed to the real provider), and a provider trait applied directly (which every
/// `IsProviderFor` node already stands for).
pub(crate) fn label_for<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitPredicate<'tcx>,
    context: Ty<'tcx>,
    names: &ComponentNameMap,
) -> Option<String> {
    let trait_ref = pred.skip_binder().trait_ref;
    let did = trait_ref.def_id;

    if is_cgp_item(tcx, did, CAN_USE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE) {
        let consumer = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.consumer);
        // `CanUseComponent<Marker, Params>` — the component's extra parameters, reattached so a
        // generic component's consumer trait reads as written (`CanCalculateArea<u32, u64, bool>`).
        let generics = render_params(trait_ref.args.type_at(2));
        Some(format!(
            "consumer trait impl `{consumer}{generics}` for context `{}`",
            trait_ref.self_ty()
        ))
    } else if is_cgp_item(tcx, did, IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE) {
        // Drop the routing `IsProviderFor` for the context itself; keep the real providers.
        if trait_ref.self_ty() == context {
            return None;
        }
        let provider = trait_ref.self_ty().to_string();
        let provider_trait = marker_role(tcx, trait_ref.args.type_at(1), names, |n| n.provider);
        // `IsProviderFor<Provider, Marker, Context, Params>` — the third argument is the context,
        // the fourth the component's extra parameters (reattached to the provider trait).
        let provider_context = trait_ref.args.type_at(2);
        let generics = render_params(trait_ref.args.type_at(3));
        Some(format!(
            "provider trait impl `{provider_trait}{generics}` with context `{provider_context}` for provider `{provider}`"
        ))
    } else if is_cgp_item(tcx, did, HAS_FIELD_TRAIT, CGP_FIELD_CRATE) {
        let field = decode_symbol(tcx, trait_ref.args.type_at(1))?;
        Some(format!(
            "field trait impl `HasField` with field `{field}` for `{}`",
            trait_ref.self_ty()
        ))
    } else if is_cgp_item(tcx, did, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)
        || is_provider_trait(tcx, did)
        || is_namespace_lookup_trait(tcx, did)
    {
        // A namespace lookup trait node (the failed `Path: DefaultNamespace<Ctx>` redirect) is pure
        // plumbing here — its `MissingRedirectWiring` leaf lead already names the path and context —
        // so drop it from the tree, as the `DelegateComponent` wiring node is dropped.
        None
    } else {
        Some(format!(
            "trait impl `{}` for `{}`",
            tcx.item_name(did),
            trait_ref.self_ty()
        ))
    }
}

/// Fold a path's rendered labels into a single-spine dependency tree, root first.
pub(crate) fn spine(labels: Vec<String>) -> Option<DependencyTree> {
    let mut rev = labels.into_iter().rev();
    let mut node = DependencyTree::leaf(rev.next()?);
    for label in rev {
        node = DependencyTree::node(label, vec![node]);
    }
    Some(node)
}

/// Render a component's extra type parameters as a trait generic list — `<u32, u64, bool>`, or the
/// empty string when there are none. CGP groups the parameters into the `Params` slot of
/// `CanUseComponent`/`IsProviderFor`: none as the unit `()`, a single one bare, several as a tuple,
/// so a tuple is unwrapped and a bare parameter reattached directly.
pub(crate) fn render_params(params: Ty<'_>) -> String {
    match params.kind() {
        ty::Tuple(elems) if elems.is_empty() => String::new(),
        ty::Tuple(elems) => format!(
            "<{}>",
            elems
                .iter()
                .map(|elem| elem.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => format!("<{params}>"),
    }
}

/// Resolve a component marker type to its consumer or provider trait name through the name map,
/// keyed by the marker's *full path* so two same-named markers in different modules never collide.
/// Falls back to the marker's bare item name when the component is not in the map, and to the
/// marker's printed form when it is not an ADT at all.
pub(crate) fn marker_role(
    tcx: TyCtxt<'_>,
    marker: Ty<'_>,
    names: &ComponentNameMap,
    role: impl Fn(ComponentTraitNames) -> String,
) -> String {
    let ty::Adt(def, _) = marker.kind() else {
        return marker.to_string();
    };
    match names.get_by_path(&tcx.def_path_str(def.did())) {
        Some(entry) => role(entry),
        None => tcx.item_name(def.did()).to_string(),
    }
}
