//! Which traits are CGP's extensible-data traits, and which of their obligations are plumbing.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::{
    CGP_BASE_TYPES_CRATE, CGP_FIELD_CRATE, CGP_FIELD_EXTRA_CRATE, CONS_TYPE, DATA_DERIVES,
    EITHER_TYPE, FIELD_ACCESS_TRAITS, NIL_TYPE, VOID_TYPE,
};
use crate::resolve::query::is_cgp_item;

/// Whether `did` is one of CGP's extensible-data traits: a trait the `cgp-field` or
/// `cgp-field-extra` crate defines (the builders, extractors, casts, and their optional and
/// defaulted forms). The walk descends into such a trait's impl, so a data operation's failure
/// reaches the presence-marker bound behind it.
pub(crate) fn is_data_trait(tcx: TyCtxt<'_>, did: DefId) -> bool {
    matches!(
        tcx.crate_name(did.krate).as_str(),
        CGP_FIELD_CRATE | CGP_FIELD_EXTRA_CRATE
    )
}

/// Whether `did` is one of CGP's extensible-data *operations*: a [data trait](is_data_trait) other
/// than the field-access traits (`HasField` and its kin), which read or write one field of a value
/// rather than operate on a whole record or enum. A call to one is what the data-call anchor takes.
pub(crate) fn is_data_operation(tcx: TyCtxt<'_>, did: DefId) -> bool {
    is_data_trait(tcx, did)
        && !FIELD_ACCESS_TRAITS
            .iter()
            .any(|name| is_cgp_item(tcx, did, name, CGP_FIELD_CRATE))
}

/// Whether `pred` is a data trait's recursion over a type-level list: an obligation of a data trait
/// whose `Self` is a `Cons`, `Nil`, `Either`, or `Void` cell, such as `CanBuildFrom`'s
/// `FieldsBuilder` over a record's field list or a cast's `FieldsExtractor` over an enum's variants.
/// The recursion is how the operation visits each field in turn, not a step the reader took, so its
/// hops are dropped from the tree and the field or variant it reaches is named by the leaf.
pub(crate) fn is_data_list_recursion<'tcx>(
    tcx: TyCtxt<'tcx>,
    pred: ty::PolyTraitClause<'tcx>,
) -> bool {
    let trait_ref = pred.skip_binder().trait_ref;
    if !is_data_trait(tcx, trait_ref.def_id) {
        return false;
    }
    let ty::Adt(def, _) = trait_ref.self_ty().kind() else {
        return false;
    };
    let did = def.did();
    is_cgp_item(tcx, did, CONS_TYPE, CGP_BASE_TYPES_CRATE)
        || is_cgp_item(tcx, did, NIL_TYPE, CGP_BASE_TYPES_CRATE)
        || is_cgp_item(tcx, did, EITHER_TYPE, CGP_FIELD_CRATE)
        || is_cgp_item(tcx, did, VOID_TYPE, CGP_FIELD_CRATE)
}

/// The derive that implements the data trait `did`, when `did` is one only a derive implements
/// (`HasFields`, `HasExtractor`, `HasBuilder`, `FromVariant`, and their borrowed forms), so a type
/// lacking it is missing that derive.
pub(crate) fn data_derive(tcx: TyCtxt<'_>, did: DefId) -> Option<&'static str> {
    DATA_DERIVES
        .iter()
        .find(|(trait_name, _)| is_cgp_item(tcx, did, trait_name, CGP_FIELD_CRATE))
        .map(|&(_, derive)| derive)
}
