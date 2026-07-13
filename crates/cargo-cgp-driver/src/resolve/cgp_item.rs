//! DefId-anchored recognition of the CGP traits and types the resolver walks.
//!
//! Every stage of the resolution is anchored by `DefId` to the CGP crate that defines the
//! trait or type it matches, so a same-named item from an unrelated crate can never drive a
//! replacement — the same discipline [`component_map`](crate::component_map) uses for
//! `IsProviderFor`.

use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, IS_PROVIDER_FOR_TRAIT};

/// Whether `def_id` is a trait/type named `name` defined by crate `krate` — the DefId anchor
/// that keeps a same-named item from an unrelated crate from driving resolution, exactly as
/// `component_map::is_cgp_is_provider_for` does for `IsProviderFor`.
pub(crate) fn is_cgp_item(tcx: TyCtxt<'_>, def_id: DefId, name: &str, krate: &str) -> bool {
    tcx.item_name(def_id).as_str() == name && tcx.crate_name(def_id.krate).as_str() == krate
}

/// The `DefId` of the CGP trait named `name` defined by crate `krate`, or `None` if the crate does
/// not use CGP. Anchored by name *and* crate, like every other CGP lookup here.
pub(crate) fn find_cgp_trait(tcx: TyCtxt<'_>, name: &str, krate: &str) -> Option<DefId> {
    tcx.all_traits_including_private()
        .find(|&did| is_cgp_item(tcx, did, name, krate))
}

/// Whether `def_id` is a CGP *provider* trait — one carrying an `IsProviderFor` supertrait. A
/// bare provider-trait obligation (`Ctx: SomeProvider<Ctx>`) is redundant with the
/// `IsProviderFor` node that stands for the same step, so the tree drops it; and the descent
/// treats a provider-trait bound as a step to walk into rather than a leaf.
pub(crate) fn is_provider_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.explicit_super_predicates_of(def_id)
        .skip_binder()
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|tp| is_cgp_item(tcx, tp.def_id(), IS_PROVIDER_FOR_TRAIT, CGP_COMPONENT_CRATE))
}

/// Decode a CGP `Symbol!` type into its string, by walking the `Chars<'c', Tail>` spine and
/// reading each `char` const argument until `Nil`. Anchored to `cgp_base_types`, and returns
/// `None` for any type that is not a well-formed `Symbol`.
pub(crate) fn decode_symbol(tcx: TyCtxt<'_>, ty: Ty<'_>) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !is_cgp_item(tcx, def.did(), "Symbol", CGP_BASE_TYPES_CRATE) {
        return None;
    }

    // `Symbol<const LEN, Chars>` — the second argument is the head of the `Chars` spine.
    let mut current = args.type_at(1);
    let mut name = String::new();
    loop {
        let ty::Adt(def, args) = current.kind() else {
            return None;
        };
        if is_cgp_item(tcx, def.did(), "Nil", CGP_BASE_TYPES_CRATE) {
            break;
        }
        if !is_cgp_item(tcx, def.did(), "Chars", CGP_BASE_TYPES_CRATE) {
            return None;
        }

        // `Chars<const CHAR: char, Tail>` — read the char, then follow the tail.
        let scalar = args.const_at(0).try_to_value()?.valtree.try_to_leaf()?;
        name.push(char::from_u32(scalar.to_u32())?);
        current = args.type_at(1);
    }
    Some(name)
}
