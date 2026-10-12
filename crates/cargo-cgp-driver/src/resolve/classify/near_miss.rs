//! Finding a near miss for a missing field or wiring key: a similar name the program wrote.

use cargo_cgp_error_processing::{WiringKey, similar_key, similar_name};
use rustc_middle::ty::{self, Ty, TyCtxt};

use crate::config::{CGP_COMPONENT_CRATE, DELEGATE_COMPONENT_TRAIT};
use crate::resolve::cgp_item::find_cgp_trait;
use crate::resolve::conflict::describe_key;

/// A named field of `owner`'s struct whose name is a likely slip of the missing `field`, such as
/// `first_name` for `frist_name`. `None` when `owner` is not a struct or carries no such field.
pub(crate) fn similar_field(owner: Ty<'_>, field: &str) -> Option<String> {
    let ty::Adt(def, _) = owner.kind() else {
        return None;
    };
    if !def.is_struct() {
        return None;
    }
    let names: Vec<String> = def
        .non_enum_variant()
        .fields
        .iter()
        .map(|field| field.name.to_string())
        .collect();
    similar_name(field, names.iter().map(String::as_str)).map(str::to_owned)
}

/// A key `context` wires that is a likely slip of `target`, a component name or an `@`-path such
/// as `@App.GreeterComponent` for `@app.GreeterComponent`. The candidates are the context's own
/// `DelegateComponent` entries, each key rendered as its entry wrote it.
pub(crate) fn similar_wired_key<'tcx>(
    tcx: TyCtxt<'tcx>,
    context: Ty<'tcx>,
    target: &str,
) -> Option<String> {
    let ty::Adt(context_def, _) = context.kind() else {
        return None;
    };
    let delegate_component = find_cgp_trait(tcx, DELEGATE_COMPONENT_TRAIT, CGP_COMPONENT_CRATE)?;
    let keys: Vec<String> = tcx
        .all_impls(delegate_component)
        .filter_map(|impl_did| {
            let trait_ref = tcx
                .impl_trait_ref(impl_did)
                .instantiate_identity()
                .skip_norm_wip();
            match trait_ref.self_ty().kind() {
                ty::Adt(def, _) if def.did() == context_def.did() => {}
                _ => return None,
            }
            match describe_key(tcx, trait_ref.args.type_at(1), impl_did)? {
                WiringKey::Component(name) => Some(name),
                WiringKey::Path(path) => Some(path),
                WiringKey::Type(_) | WiringKey::Blanket(_) => None,
            }
        })
        .collect();
    similar_key(target, keys.iter().map(String::as_str)).map(str::to_owned)
}
