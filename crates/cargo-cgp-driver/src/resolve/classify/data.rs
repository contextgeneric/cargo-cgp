//! Classifying an unmet extensible-data obligation by the presence markers it reads.

use cargo_cgp_error_processing::{DeriveFix, FieldFix, FieldIssue, Leaf, NearMiss, PartialKind};
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CGP_FIELD_CRATE, EXTRACT_FIELD_TRAIT, FINALIZE_BUILD_TRAIT, FINALIZE_EXTRACT_TRAIT,
    FROM_VARIANT_TRAIT, MAPPER_ASSOC, UPDATE_FIELD_TRAIT, VALUE_ASSOC,
};
use crate::resolve::cgp_item::{decode_index, decode_symbol, is_cgp_item};
use crate::resolve::classify::{owner_has_impl_of, projected_type, similar_field, similar_variant};
use crate::resolve::data::{Marker, PartialRead, data_derive, is_data_trait, marker, read_partial};
use crate::resolve::walk::ProjectionMismatch;

/// The data leaf for an unmet obligation of one of CGP's extensible-data traits, or `None` when the
/// obligation is not one this leaf words. A builder's or an extractor's partial value records each
/// field's or variant's state in a marker, so the obligation names the field or variant at fault:
///
/// - `UpdateField<Tag, M>` on a builder: with no impl, the struct lacks the field (the missing-field
///   leaf); with its `Mapper` found set where the update needs it unset, the field is already set;
///   found unset where the update needs it set, the field is not set.
/// - `FinalizeBuild` on a builder: the fields it does not hold are not set.
/// - `ExtractField<Tag>` on an extractor with no impl: the enum lacks the variant.
/// - `FinalizeExtract` on an extractor: the variants it may still yield are not extracted.
/// - `FromVariant<Tag>` on an enum: with no impl, the enum lacks the variant (or derives no
///   `FromVariant` at all); with its `Value` found other than required, the variant's payload has
///   the wrong type.
/// - a trait only a data derive implements (`HasFields`, `HasExtractor`, `HasBuilder`, …), unmet on
///   a type that is not a partial value: the type does not implement it, and a `help` names the
///   derive when it can be added there.
pub(crate) fn classify_data_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
) -> Option<Leaf> {
    let did = leaf_ref.def_id;
    if !is_data_trait(tcx, did) {
        return None;
    }
    let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
    let is_trait = |name: &str| is_cgp_item(tcx, did, name, CGP_FIELD_CRATE);
    let mismatched = |assoc: &str| {
        mismatch.filter(|mismatch| tcx.item_name(mismatch.assoc_did).as_str() == assoc)
    };

    if is_trait(UPDATE_FIELD_TRAIT) {
        let partial = builder(tcx, self_ty)?;
        let name = tag_name(tcx, leaf_ref.args.type_at(1))?;
        let owner = partial.target.to_string();
        return match (mismatch, mismatched(MAPPER_ASSOC)) {
            // Adding the field to the struct is not offered: the operation then builds it, but a
            // field the struct needs from elsewhere may still be unset, as when a merged source
            // carries a renamed field.
            (None, _) if partial.marker_of(&name).is_none() => Some(Leaf::Field {
                fix: FieldFix {
                    confirmed: false,
                    ty: None,
                    similar: similar_field(partial.target, &name),
                },
                name,
                owner,
                issue: FieldIssue::Missing,
            }),
            (_, Some(mapper)) => match (marker(tcx, mapper.expected), partial.marker_of(&name)?) {
                (Marker::Nothing, Marker::Present) => Some(Leaf::FieldAlreadySet { name, owner }),
                (Marker::Present, Marker::Nothing) => Some(Leaf::FieldsNotSet {
                    names: vec![name],
                    owner,
                }),
                _ => None,
            },
            _ => None,
        };
    }
    if is_trait(FINALIZE_BUILD_TRAIT) && mismatch.is_none() {
        let partial = builder(tcx, self_ty)?;
        let names: Vec<String> = partial
            .members
            .iter()
            .filter(|(_, marker)| *marker != Marker::Present)
            .map(|(name, _)| name.clone())
            .collect();
        return (!names.is_empty()).then(|| Leaf::FieldsNotSet {
            names,
            owner: partial.target.to_string(),
        });
    }
    if is_trait(EXTRACT_FIELD_TRAIT) && mismatch.is_none() {
        let partial = extractor(tcx, self_ty)?;
        let name = tag_name(tcx, leaf_ref.args.type_at(1))?;
        return partial
            .marker_of(&name)
            .is_none()
            .then(|| Leaf::MissingVariant {
                similar: NearMiss(similar_variant(partial.target, &name)),
                name,
                owner: partial.target.to_string(),
            });
    }
    if is_trait(FINALIZE_EXTRACT_TRAIT) && mismatch.is_none() {
        let partial = extractor(tcx, self_ty)?;
        let names = partial.with_marker(Marker::Present);
        return (!names.is_empty()).then(|| Leaf::VariantsNotExtracted {
            names,
            owner: partial.target.to_string(),
        });
    }
    if is_trait(FROM_VARIANT_TRAIT) && read_partial(tcx, self_ty).is_none() {
        let name = tag_name(tcx, leaf_ref.args.type_at(1))?;
        let owner = self_ty.to_string();
        if let Some(value) = mismatched(VALUE_ASSOC) {
            return Some(Leaf::VariantTypeMismatch {
                name,
                owner,
                expected: value.expected.to_string(),
                actual: projected_type(tcx, value.alias)
                    .map(|ty| ty.to_string())
                    .unwrap_or_else(|| "_".to_owned()),
            });
        }
        if mismatch.is_some() {
            return None;
        }
        // An enum that derives `FromVariant` implements it for each of its variants, so a missing
        // impl for this tag is a variant the enum lacks; one with no impl at all lacks the derive.
        if owner_has_impl_of(tcx, did, self_ty) {
            return Some(Leaf::MissingVariant {
                similar: NearMiss(similar_variant(self_ty, &name)),
                name,
                owner,
            });
        }
    }
    let derive = data_derive(tcx, did)?;
    if mismatch.is_some() || read_partial(tcx, self_ty).is_some() {
        return None;
    }
    Some(Leaf::DataTraitUnimplemented {
        owner: self_ty.to_string(),
        trait_name: tcx.item_name(did).to_string(),
        derive: DeriveFix(derive_applies(self_ty, derive).then(|| derive.to_owned())),
    })
}

/// `ty` read as a record builder.
fn builder<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<PartialRead<'tcx>> {
    read_partial(tcx, ty).filter(|partial| partial.kind == PartialKind::Builder)
}

/// `ty` read as an enum extractor, owning or borrowing its payloads.
fn extractor<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<PartialRead<'tcx>> {
    read_partial(tcx, ty).filter(|partial| partial.kind != PartialKind::Builder)
}

/// The field or variant a tag names: the string of a `Symbol!` tag, or the position of an `Index`
/// tag (a tuple field's name).
fn tag_name<'tcx>(tcx: TyCtxt<'tcx>, tag: Ty<'tcx>) -> Option<String> {
    decode_symbol(tcx, tag).or_else(|| decode_index(tcx, tag).map(|index| index.to_string()))
}

/// Whether `derive` can be added to `ty`: a struct or enum the crate defines, of the kind the
/// derive accepts (`ExtractField` and `FromVariant` an enum, `BuildField` a struct, `HasFields`
/// either).
fn derive_applies(ty: Ty<'_>, derive: &str) -> bool {
    let ty::Adt(def, _) = ty.kind() else {
        return false;
    };
    if !def.did().is_local() {
        return false;
    }
    match derive {
        "ExtractField" | "FromVariant" => def.is_enum(),
        "BuildField" => def.is_struct(),
        _ => def.is_struct() || def.is_enum(),
    }
}
