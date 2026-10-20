//! Detecting a `#[use_type]` import that misspells the associated type it imports.
//!
//! `#[use_type(HasScalarType.Scalr)]` rewrites each bare `Scalr` in a CGP definition to
//! `<Self as HasScalarType>::Scalr`, keeping the span of the alias the program wrote, so name
//! resolution reports the missing associated type (`E0576`) at that alias. The compiler's help
//! respells the alias, while the misspelling is in the import. This module confirms that the path
//! came from such an import and finds the associated type the name is a slip of.

use cargo_cgp_error_processing::{UseTypeMisspelling, similar_name};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;

use crate::resolve::unimported::enclosing_cgp_item;

/// Recognize an `E0576` "cannot find associated type `type_name` in trait `trait_name`" at
/// `primary_span` as a misspelled `#[use_type]` import. `None` unless the caret sits in an item a
/// CGP macro generated and the source writes the name bare there (a path the program qualified
/// itself, `<Self as Trait>::Name`, is respelled where it stands, as the compiler's help says),
/// exactly one trait has the reported name, and it declares an associated type the name is a slip
/// of.
pub fn detect_use_type_misspelling(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    type_name: &str,
    trait_name: &str,
) -> Option<UseTypeMisspelling> {
    let source_map = tcx.sess.source_map();
    let written_before = source_map.span_to_prev_source(primary_span).ok()?;
    if written_before.trim_end().ends_with("::") {
        return None;
    }
    let alias = source_map.span_to_snippet(primary_span).ok()?;
    enclosing_cgp_item(tcx, primary_span)?;
    let similar = similar_associated_type(tcx, type_name, trait_name)?;
    Some(UseTypeMisspelling {
        type_name: type_name.to_owned(),
        trait_name: trait_name.to_owned(),
        similar,
        alias_is_name: alias == type_name,
    })
}

/// The associated type of the one trait named `trait_name` that `type_name` is a slip of.
fn similar_associated_type(tcx: TyCtxt<'_>, type_name: &str, trait_name: &str) -> Option<String> {
    let name = trait_name.rsplit("::").next()?;
    let mut traits = tcx
        .all_traits_including_private()
        .filter(|&trait_did| tcx.item_name(trait_did).as_str() == name);
    let trait_did = traits.next()?;
    if traits.next().is_some() {
        return None;
    }
    let declared: Vec<String> = tcx
        .associated_items(trait_did)
        .in_definition_order()
        .filter(|item| item.kind.tag() == ty::AssocTag::Type)
        .map(|item| item.name().to_string())
        .collect();
    similar_name(type_name, declared.iter().map(String::as_str)).map(str::to_owned)
}
