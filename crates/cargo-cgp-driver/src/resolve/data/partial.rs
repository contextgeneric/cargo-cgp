//! Reading a partial value's presence markers back into the fields or variants they track.

use cargo_cgp_error_processing::{PartialKind, PartialValue};
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{
    CGP_FIELD_CRATE, IS_MUT_TYPE, IS_NOTHING_TYPE, IS_OPTIONAL_TYPE, IS_PRESENT_TYPE, IS_REF_TYPE,
    IS_VOID_TYPE, MAP_TYPE_TRAIT, PARTIAL_DATA_TARGET, PARTIAL_DATA_TRAIT,
};
use crate::resolve::cgp_item::{find_cgp_trait, is_cgp_item};
use crate::resolve::query::{owner_has_impl_of, projected_type};

/// The state a presence marker records for one field or variant of a partial value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Marker {
    /// `IsPresent`: a builder holds the field, or an extractor may still yield the variant.
    Present,
    /// `IsNothing`: a builder does not hold the field yet.
    Nothing,
    /// `IsVoid`: an extractor has ruled the variant out.
    Void,
    /// `IsOptional`: an optional builder holds the field as an `Option`.
    Optional,
    /// A marker of the program's own, or a generic parameter.
    Other,
}

/// A partial value read back: what it is, the struct or enum it builds or matches, and the state of
/// each field (a builder) or variant (an extractor) in declaration order.
pub(crate) struct PartialRead<'tcx> {
    pub(crate) kind: PartialKind,
    pub(crate) target: Ty<'tcx>,
    pub(crate) members: Vec<(String, Marker)>,
}

impl PartialRead<'_> {
    /// The members whose marker is `marker`, in declaration order.
    pub(crate) fn with_marker(&self, marker: Marker) -> Vec<String> {
        self.members
            .iter()
            .filter(|(_, member)| *member == marker)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// The marker of the member named `name`, or `None` when the target has no such member.
    pub(crate) fn marker_of(&self, name: &str) -> Option<Marker> {
        self.members
            .iter()
            .find(|(member, _)| member == name)
            .map(|&(_, marker)| marker)
    }

    /// The value in the rustc-free form a chain hop names it by.
    pub(crate) fn value(&self) -> PartialValue {
        let marked = match self.kind {
            PartialKind::Builder => Marker::Present,
            _ => Marker::Void,
        };
        PartialValue {
            kind: self.kind,
            target: self.target.to_string(),
            marked: self.with_marker(marked),
            optional: match self.kind {
                PartialKind::Builder => self.with_marker(Marker::Optional),
                _ => Vec::new(),
            },
        }
    }
}

/// Read `ty` as a partial value, when it is one: an ADT implementing `PartialData` (the impl a data
/// derive generates for each partial record and enum), whose `Target` names the struct or enum it
/// builds or matches. Each field of a partial record, and the payload of each variant of a partial
/// enum, is stored as `<M as MapType>::Map<…>` for one of the ADT's marker parameters, so the ADT's
/// own definition pairs every marker with the field or variant it tracks, and `ty`'s arguments give
/// each marker's state. `None` for any other type.
pub(crate) fn read_partial<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<PartialRead<'tcx>> {
    let ty = tcx.erase_and_anonymize_regions(ty);
    let ty::Adt(adt, args) = ty.kind() else {
        return None;
    };
    // Pair each field or variant with its marker first: the check is cheap and rules out almost
    // every type, so the trait lookups below run only for a likely partial value. A partial enum
    // stores each variant's payload in its one field, so the variant names it.
    let mut stored: Vec<(&ty::VariantDef, &ty::FieldDef, u32)> = Vec::new();
    for variant in adt.variants() {
        for field in &variant.fields {
            let field_ty = tcx
                .type_of(field.did)
                .instantiate_identity()
                .skip_norm_wip();
            if let Some(index) = map_type_param(tcx, field_ty) {
                stored.push((variant, field, index));
            }
        }
    }
    if stored.is_empty() {
        return None;
    }

    let partial_data = find_cgp_trait(tcx, PARTIAL_DATA_TRAIT, CGP_FIELD_CRATE)?;
    if !owner_has_impl_of(tcx, partial_data, ty) {
        return None;
    }
    let target_did = tcx
        .associated_items(partial_data)
        .in_definition_order()
        .find(|item| tcx.item_name(item.def_id).as_str() == PARTIAL_DATA_TARGET)?
        .def_id;
    let target = projected_type(
        tcx,
        Ty::new_projection(tcx, ty::IsRigid::No, target_did, [ty]),
    )?;
    let target_adt = target.ty_adt_def()?;

    let members = stored
        .into_iter()
        .map(|(variant, field, index)| {
            let name = if target_adt.is_enum() {
                variant.name
            } else {
                field.name
            };
            (name.to_string(), marker(tcx, args.type_at(index as usize)))
        })
        .collect();

    let kind = if target_adt.is_struct() {
        PartialKind::Builder
    } else if args.types().any(|arg| is_marker(tcx, arg, IS_REF_TYPE)) {
        PartialKind::RefExtractor
    } else if args.types().any(|arg| is_marker(tcx, arg, IS_MUT_TYPE)) {
        PartialKind::MutExtractor
    } else {
        PartialKind::Extractor
    };
    Some(PartialRead {
        kind,
        target,
        members,
    })
}

/// The index of the marker parameter a partial value's field type is stored under: `i` for
/// `<__Fi__ as MapType>::Map<…>`, anchored to `cgp-field`'s `MapType`.
fn map_type_param<'tcx>(tcx: TyCtxt<'tcx>, field_ty: Ty<'tcx>) -> Option<u32> {
    let ty::Alias(_, alias) = field_ty.kind() else {
        return None;
    };
    let ty::AliasTyKind::Projection { def_id } = alias.kind else {
        return None;
    };
    if !is_cgp_item(tcx, tcx.parent(def_id), MAP_TYPE_TRAIT, CGP_FIELD_CRATE) {
        return None;
    }
    match alias.args.type_at(0).kind() {
        ty::Param(param) => Some(param.index),
        _ => None,
    }
}

/// The state a marker argument records.
pub(crate) fn marker<'tcx>(tcx: TyCtxt<'tcx>, arg: Ty<'tcx>) -> Marker {
    if is_marker(tcx, arg, IS_PRESENT_TYPE) {
        Marker::Present
    } else if is_marker(tcx, arg, IS_NOTHING_TYPE) {
        Marker::Nothing
    } else if is_marker(tcx, arg, IS_VOID_TYPE) {
        Marker::Void
    } else if is_marker(tcx, arg, IS_OPTIONAL_TYPE) {
        Marker::Optional
    } else {
        Marker::Other
    }
}

/// Whether `ty` is the `cgp-field` marker type named `name`.
fn is_marker(tcx: TyCtxt<'_>, ty: Ty<'_>, name: &str) -> bool {
    matches!(ty.kind(), ty::Adt(def, _) if is_cgp_item(tcx, def.did(), name, CGP_FIELD_CRATE))
}
