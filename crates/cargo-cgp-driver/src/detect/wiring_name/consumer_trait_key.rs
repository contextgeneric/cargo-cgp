//! Recognizing a consumer trait written as a wiring key, as the compiler lowers the entry
//! (`E0782`).

use cargo_cgp_error_processing::UnresolvedWiringName;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{QPath, TyKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::detect::wiring_name::component::consumer_component;
use crate::detect::wiring_name::slot::{NameSlot, name_slot_at};
use crate::resolve::query::is_consumer_trait;

/// Recognize an `E0782` at `span` as a consumer trait written as a wiring key: the span is the key
/// of a wiring entry, the key resolves to a CGP consumer trait, and that trait has a component.
pub fn detect_consumer_trait_as_key(tcx: TyCtxt<'_>, span: Span) -> Option<UnresolvedWiringName> {
    let NameSlot::Key { key } = name_slot_at(tcx, span)? else {
        return None;
    };
    // A bare trait in a type position lowers to a trait object, or to a path in an edition that
    // still accepts one.
    let res = match key.kind {
        TyKind::TraitObject([bound], _) => bound.trait_ref.path.res,
        TyKind::Path(QPath::Resolved(None, path)) => path.res,
        _ => return None,
    };
    let Res::Def(DefKind::Trait, consumer_did) = res else {
        return None;
    };
    if !is_consumer_trait(tcx, consumer_did) {
        return None;
    }
    let component_did = consumer_component(tcx, consumer_did)?;
    Some(UnresolvedWiringName::ConsumerTraitAsKey {
        consumer: tcx.item_name(consumer_did).to_string(),
        component: tcx.item_name(component_did).to_string(),
    })
}
