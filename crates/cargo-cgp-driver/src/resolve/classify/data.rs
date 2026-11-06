//! Classifying an unmet extensible-data obligation by the presence markers it reads.

use cargo_cgp_error_processing::{
    DeriveFix, FieldFix, FieldIssue, Leaf, MethodFix, NearMiss, PartialKind, SuggestedCall,
};
use rustc_middle::ty::{self, Ty, TyCtxt, Upcast as _};
use rustc_span::def_id::DefId;

use crate::config::{
    BUILD_FIELD_DERIVE, CGP_FIELD_CRATE, CGP_FIELD_EXTRA_CRATE, EXTRACT_FIELD_DERIVE,
    EXTRACT_FIELD_TRAIT, FINALIZE_BUILD_TRAIT, FINALIZE_EXTRACT_TRAIT, FROM_VARIANT_DERIVE,
    FROM_VARIANT_TRAIT, MAPPER_ASSOC, PARTIAL_VALUE_OPERATIONS, TO_OPTIONAL_TRAIT,
    UPDATE_FIELD_TRAIT, VALUE_ASSOC,
};
use crate::resolve::classify::{similar_field, similar_variant};
use crate::resolve::data::{Marker, PartialRead, data_derive, is_data_trait, marker, read_partial};
use crate::resolve::query::{
    ProjectionMismatch, decode_index, decode_symbol, find_cgp_trait, holds, import_path,
    is_cgp_item, owner_has_impl_of, projected_type,
};

/// The data leaf for an unmet obligation of one of CGP's extensible-data traits, or `None` when the
/// obligation is not one this leaf words. A builder's or an extractor's partial value records each
/// field's or variant's state in a marker, so the obligation names the field or variant at fault:
///
/// - `UpdateField<Tag, M>` on a builder: with no impl, the struct lacks the field (the missing-field
///   leaf); with its `Mapper` found set where the update needs it unset, the field is already set;
///   found unset where the update needs it set, the field is not set; found set or unset where an
///   optional builder's update needs it optional, the field is not optional.
/// - `FinalizeBuild` on a builder: the fields it does not hold are not set.
/// - `ExtractField<Tag>` on an extractor with no impl: the enum lacks the variant, or the extractor
///   has already ruled it out.
/// - `FinalizeExtract` on an extractor: the variants it may still yield are not extracted.
/// - `FromVariant<Tag>` on an enum: with no impl, the enum lacks the variant (or derives no
///   `FromVariant` at all); with its `Value` found other than required, the variant's payload has
///   the wrong type.
/// - a trait only a data derive implements (`HasFields`, `HasExtractor`, `HasBuilder`, …): unmet on
///   a type that is not a partial value, the type does not implement it, and a `help` names the
///   derive when it can be added there; unmet on a partial value, the operation was given a builder
///   or an extractor where the struct or enum belongs, and a `help` names the method a partial value
///   takes instead when its `parent` operation has one that holds.
pub fn classify_data_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
) -> Option<Leaf> {
    let did = leaf_ref.def_id;
    if !is_data_trait(tcx, did) {
        return None;
    }
    let self_ty = tcx.erase_and_anonymize_regions(leaf_ref.self_ty());
    let is_trait = |name: &str| is_cgp_item(tcx, did, name, CGP_FIELD_CRATE);

    if is_trait(UPDATE_FIELD_TRAIT) {
        return update_field_leaf(tcx, leaf_ref, self_ty, mismatch);
    }
    if is_trait(FINALIZE_BUILD_TRAIT) && mismatch.is_none() {
        return finalize_build_leaf(tcx, self_ty);
    }
    if is_trait(EXTRACT_FIELD_TRAIT) && mismatch.is_none() {
        return extract_field_leaf(tcx, leaf_ref, self_ty);
    }
    if is_trait(FINALIZE_EXTRACT_TRAIT) && mismatch.is_none() {
        return finalize_extract_leaf(tcx, self_ty);
    }
    if is_trait(FROM_VARIANT_TRAIT) && read_partial(tcx, self_ty).is_none() {
        return from_variant_leaf(tcx, leaf_ref, self_ty, parent, mismatch);
    }
    derive_leaf(tcx, did, self_ty, parent, mismatch)
}

/// The leaf for an unmet `UpdateField<Tag, M>` on a builder: a field the struct lacks, or one whose
/// `Mapper` was found set, unset, or not optional where the update needs it otherwise.
fn update_field_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    self_ty: Ty<'tcx>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
) -> Option<Leaf> {
    let partial = builder(tcx, self_ty)?;
    let name = tag_name(tcx, leaf_ref.args.type_at(1))?;
    let owner = partial.target.to_string();
    match (mismatch, mismatched(tcx, mismatch, MAPPER_ASSOC)) {
        // Adding the field to the struct is not offered: the operation then builds it, but a
        // field the struct needs from elsewhere may still be unset, as when a merged source
        // carries a renamed field.
        (None, _) if partial.marker_of(&name).is_none() => Some(Leaf::Field {
            fix: FieldFix {
                confirmed: false,
                ty: None,
                similar: similar_field(partial.target, &name),
                index_tag: None,
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
            (Marker::Optional, Marker::Nothing | Marker::Present) => Some(Leaf::FieldNotOptional {
                name,
                owner,
                convert: MethodFix(to_optional(tcx, self_ty)),
            }),
            _ => None,
        },
        _ => None,
    }
}

/// The leaf for an unmet `FinalizeBuild` on a builder: the fields it does not hold are not set.
fn finalize_build_leaf<'tcx>(tcx: TyCtxt<'tcx>, self_ty: Ty<'tcx>) -> Option<Leaf> {
    let partial = builder(tcx, self_ty)?;
    let names: Vec<String> = partial
        .members
        .iter()
        .filter(|(_, marker)| *marker != Marker::Present)
        .map(|(name, _)| name.clone())
        .collect();
    (!names.is_empty()).then(|| Leaf::FieldsNotSet {
        names,
        owner: partial.target.to_string(),
    })
}

/// The leaf for an unmet `ExtractField<Tag>` on an extractor: the enum lacks the variant, or the
/// extractor has already ruled it out.
fn extract_field_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    self_ty: Ty<'tcx>,
) -> Option<Leaf> {
    let partial = extractor(tcx, self_ty)?;
    let name = tag_name(tcx, leaf_ref.args.type_at(1))?;
    let owner = partial.target.to_string();
    match partial.marker_of(&name) {
        None => Some(Leaf::MissingVariant {
            similar: NearMiss(similar_variant(partial.target, &name)),
            name,
            owner,
        }),
        Some(Marker::Void) => Some(Leaf::VariantAlreadyExtracted { name, owner }),
        Some(_) => None,
    }
}

/// The leaf for an unmet `FinalizeExtract` on an extractor: the variants it may still yield are not
/// extracted.
fn finalize_extract_leaf<'tcx>(tcx: TyCtxt<'tcx>, self_ty: Ty<'tcx>) -> Option<Leaf> {
    let partial = extractor(tcx, self_ty)?;
    let names = partial.with_marker(Marker::Present);
    (!names.is_empty()).then(|| Leaf::VariantsNotExtracted {
        names,
        owner: partial.target.to_string(),
    })
}

/// The leaf for an unmet `FromVariant<Tag>` on an enum: the variant's payload has the wrong type, or
/// the enum lacks the variant; an enum with no `FromVariant` impl at all lacks the derive, which
/// [`derive_leaf`] words.
fn from_variant_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    leaf_ref: ty::TraitRef<'tcx>,
    self_ty: Ty<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
) -> Option<Leaf> {
    let did = leaf_ref.def_id;
    let name = tag_name(tcx, leaf_ref.args.type_at(1))?;
    let owner = self_ty.to_string();
    if let Some(value) = mismatched(tcx, mismatch, VALUE_ASSOC) {
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
    derive_leaf(tcx, did, self_ty, parent, mismatch)
}

/// The leaf for an unmet trait only a data derive implements (`HasFields`, `HasExtractor`,
/// `HasBuilder`, …): on a partial value, the operation was given a builder or an extractor where the
/// struct or enum belongs; on any other type, the type does not implement the trait. `None` when
/// the trait comes from no derive, or the branch carried an unmet projection.
fn derive_leaf<'tcx>(
    tcx: TyCtxt<'tcx>,
    did: DefId,
    self_ty: Ty<'tcx>,
    parent: Option<ty::TraitRef<'tcx>>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
) -> Option<Leaf> {
    let derive = data_derive(tcx, did)?;
    if mismatch.is_some() {
        return None;
    }
    if let Some(partial) = read_partial(tcx, self_ty) {
        return Some(Leaf::PartialLacksDataTrait {
            kind: partial.kind,
            target: partial.target.to_string(),
            trait_name: tcx.item_name(did).to_string(),
            replacement: MethodFix(
                // The operation the partial value was given to, when it is the trait the leaf is
                // required by (the downcast whose `Self` is the extractor).
                parent
                    .filter(|parent| tcx.erase_and_anonymize_regions(parent.self_ty()) == self_ty)
                    .and_then(|parent| partial_operation(tcx, parent)),
            ),
        });
    }
    Some(Leaf::DataTraitUnimplemented {
        owner: self_ty.to_string(),
        trait_name: tcx.item_name(did).to_string(),
        derive: DeriveFix(derive_applies(self_ty, derive).then(|| derive.to_owned())),
    })
}

/// The `mismatch` when it is on the associated type named `assoc`.
fn mismatched<'tcx>(
    tcx: TyCtxt<'tcx>,
    mismatch: Option<ProjectionMismatch<'tcx>>,
    assoc: &str,
) -> Option<ProjectionMismatch<'tcx>> {
    mismatch.filter(|mismatch| tcx.item_name(mismatch.assoc_did).as_str() == assoc)
}

/// The conversion of the record builder `builder` to an optional one (`to_optional`), when the
/// builder takes it, so an optional-builder operation can follow it.
fn to_optional<'tcx>(tcx: TyCtxt<'tcx>, builder: Ty<'tcx>) -> Option<SuggestedCall> {
    let to_optional = find_cgp_trait(tcx, TO_OPTIONAL_TRAIT, CGP_FIELD_EXTRA_CRATE)?;
    let converts = ty::TraitRef::new(tcx, to_optional, [builder]);
    if !holds(tcx, ty::Binder::dummy(converts).upcast(tcx)) {
        return None;
    }
    Some(SuggestedCall {
        method: only_method(tcx, to_optional)?,
        replaces: None,
        trait_path: import_path(tcx, to_optional),
    })
}

/// The method a partial value takes in place of the operation `called` (a struct or enum value's
/// operation, called on a partial value): `downcast_fields` for `downcast` on an extractor. `None`
/// when the operation has no partial-value counterpart, or the counterpart does not hold for the
/// partial value either.
fn partial_operation<'tcx>(tcx: TyCtxt<'tcx>, called: ty::TraitRef<'tcx>) -> Option<SuggestedCall> {
    let (_, partial_name) = PARTIAL_VALUE_OPERATIONS
        .iter()
        .find(|(value_name, _)| is_cgp_item(tcx, called.def_id, value_name, CGP_FIELD_CRATE))?;
    let partial_did = find_cgp_trait(tcx, partial_name, CGP_FIELD_CRATE)?;
    let counterpart = ty::TraitRef::new(tcx, partial_did, called.args);
    if !holds(tcx, ty::Binder::dummy(counterpart).upcast(tcx)) {
        return None;
    }
    Some(SuggestedCall {
        method: only_method(tcx, partial_did)?,
        replaces: Some(only_method(tcx, called.def_id)?),
        trait_path: import_path(tcx, partial_did),
    })
}

/// The name of the one method a data trait declares, as `downcast` for `CanDowncast`.
fn only_method(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<String> {
    let mut methods = tcx
        .associated_items(trait_did)
        .in_definition_order()
        .filter(|item| matches!(item.kind, ty::AssocKind::Fn { .. }));
    let method = methods.next()?;
    methods
        .next()
        .is_none()
        .then(|| tcx.item_name(method.def_id).to_string())
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
        EXTRACT_FIELD_DERIVE | FROM_VARIANT_DERIVE => def.is_enum(),
        BUILD_FIELD_DERIVE => def.is_struct(),
        _ => def.is_struct() || def.is_enum(),
    }
}
