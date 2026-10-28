//! Detecting a `#[use_type]` import that misspells the associated type it imports.
//!
//! `#[use_type(HasScalarType.Scalr)]` rewrites each bare `Scalr` in a CGP definition to
//! `<Self as HasScalarType>::Scalr`, keeping the span of the alias the program wrote, so name
//! resolution reports the missing associated type (`E0576`) at that alias. The compiler's help
//! respells the alias, while the misspelling is in the import. This module confirms that the path
//! came from such an import and finds the associated type the import meant.

use cargo_cgp_error_processing::{UseTypeMisspelling, similar_name};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::detect::unimported::enclosing_cgp_item;

/// Recognize an `E0576` "cannot find associated type `type_name` in trait `trait_name`" at
/// `primary_span` as a misspelled `#[use_type]` import. `None` unless the caret sits in an item a
/// CGP macro generated and the source writes the name bare there (a path the program qualified
/// itself, `<Self as Trait>::Name`, is respelled where it stands, as the compiler's help says),
/// one trait of the reported name is the one meant, and the import can be taken to mean one of its
/// associated types (`meant_associated_type`). `notes` are the spans the diagnostic's notes point
/// at, which tell apart two traits that share the name.
pub fn detect_use_type_misspelling(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    type_name: &str,
    trait_name: &str,
    notes: &[Span],
) -> Option<UseTypeMisspelling> {
    let source_map = tcx.sess.source_map();
    let written_before = source_map.span_to_prev_source(primary_span).ok()?;
    if written_before.trim_end().ends_with("::") {
        return None;
    }
    let alias = source_map.span_to_snippet(primary_span).ok()?;
    enclosing_cgp_item(tcx, primary_span)?;
    let similar = meant_associated_type(tcx, type_name, trait_name, notes)?;
    Some(UseTypeMisspelling {
        type_name: type_name.to_owned(),
        trait_name: trait_name.to_owned(),
        similar,
        alias_is_name: alias == type_name,
    })
}

/// The associated type of the trait named `trait_name` that an import naming `type_name` meant: the
/// trait's only associated type, which is the one name an import of the trait can give and the one
/// the compiler suggests whatever its spelling, or else the one `type_name` is a slip of. When
/// several traits share the name, the one meant is the one whose associated type a note of the
/// diagnostic points at: the compiler's "associated type … defined here" names the trait it
/// resolved.
fn meant_associated_type(
    tcx: TyCtxt<'_>,
    type_name: &str,
    trait_name: &str,
    notes: &[Span],
) -> Option<String> {
    let name = trait_name.rsplit("::").next()?;
    let mut traits: Vec<DefId> = tcx
        .all_traits_including_private()
        .filter(|&trait_did| tcx.item_name(trait_did).as_str() == name)
        .collect();
    if traits.len() > 1 {
        traits.retain(|&trait_did| {
            tcx.associated_items(trait_did)
                .in_definition_order()
                .any(|item| {
                    let defined = tcx.def_span(item.def_id);
                    notes.iter().any(|note| note.overlaps(defined))
                })
        });
    }
    let [trait_did] = traits[..] else {
        return None;
    };
    let declared: Vec<String> = tcx
        .associated_items(trait_did)
        .in_definition_order()
        .filter(|item| item.kind.tag() == ty::AssocTag::Type)
        .map(|item| item.name().to_string())
        .collect();
    match declared.as_slice() {
        [only] => Some(only.clone()),
        _ => similar_name(type_name, declared.iter().map(String::as_str)).map(str::to_owned),
    }
}
