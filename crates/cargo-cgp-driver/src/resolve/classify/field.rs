//! Inspecting the struct a `HasField` bound lands on, and its `Deref` chain.

use cargo_cgp_error_processing::FieldIssue;
use rustc_middle::ty::{self, Ty, TyCtxt};

/// Bound on how far the `Deref` chain is followed when looking for a field, so a cyclic `Deref`
/// (`A: Deref<Target = B>`, `B: Deref<Target = A>`) cannot make the search loop.
const MAX_DEREF: u32 = 16;

/// Classify why the `HasField` bound on `owner` for `field` is unmet: whether `owner` genuinely
/// lacks the field, carries it directly (so only the `HasField` impl or its type is at fault), or
/// reaches it only through a `Deref` target the derive does not cross.
pub(crate) fn field_issue<'tcx>(tcx: TyCtxt<'tcx>, owner: Ty<'tcx>, field: &str) -> FieldIssue {
    if adt_has_field(owner, field) {
        return FieldIssue::Present;
    }
    let mut current = owner;
    for _ in 0..MAX_DEREF {
        let Some(target) = deref_target(tcx, current) else {
            break;
        };
        if adt_has_field(target, field) {
            return FieldIssue::PresentViaDeref {
                target: target.to_string(),
            };
        }
        current = target;
    }
    FieldIssue::Missing
}

/// The declared type of `ty`'s named field `field`, as a display string, or `None` when `ty` is
/// not a struct or has no such field. Read from the struct's `DefId` with the type's own generic
/// arguments substituted, so a same-named struct in another module is never queried and a generic
/// context's field type is instantiated correctly.
pub(crate) fn field_type<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>, field: &str) -> Option<String> {
    let ty::Adt(def, args) = ty.kind() else {
        return None;
    };
    if !def.is_struct() {
        return None;
    }
    let field_def = def
        .non_enum_variant()
        .fields
        .iter()
        .find(|f| f.name.as_str() == field)?;
    let field_ty = field_def.ty(tcx, args).skip_norm_wip();
    Some(tcx.erase_and_anonymize_regions(field_ty).to_string())
}

/// Whether `ty` is a struct with a named field called `field`. Only a struct can carry named
/// fields a `HasField` derive would key on, so an enum, tuple, or non-ADT is never a match.
fn adt_has_field(ty: Ty<'_>, field: &str) -> bool {
    match ty.kind() {
        ty::Adt(def, _) if def.is_struct() => def
            .non_enum_variant()
            .fields
            .iter()
            .any(|f| f.name.as_str() == field),
        _ => false,
    }
}

/// The `Deref::Target` of `ty`, read straight from the concrete `impl Deref for ty` rather than by
/// normalizing a projection, so it needs no inference context. Returns `None` when `ty` has no
/// matching `Deref` impl. Matches the impl by its `Self` type, so a generic `Deref` impl whose
/// `Self` is not exactly `ty` is skipped — sufficient for the concrete contexts a check targets.
fn deref_target<'tcx>(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let deref_trait = tcx.lang_items().deref_trait()?;
    let ty = tcx.erase_and_anonymize_regions(ty);

    for impl_did in tcx.all_impls(deref_trait) {
        let impl_self = tcx.type_of(impl_did).instantiate_identity().skip_norm_wip();
        if tcx.erase_and_anonymize_regions(impl_self) != ty {
            continue;
        }
        // The `Deref` impl's single associated type is its `Target`; its value is the target type.
        for assoc in tcx.associated_items(impl_did).in_definition_order() {
            if assoc.kind.tag() == ty::AssocTag::Type {
                let target = tcx
                    .type_of(assoc.def_id)
                    .instantiate_identity()
                    .skip_norm_wip();
                return Some(tcx.erase_and_anonymize_regions(target));
            }
        }
    }
    None
}
