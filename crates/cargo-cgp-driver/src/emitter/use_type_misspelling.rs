//! Replacing the compiler's help on a `#[use_type]` import that misspells its associated type.
//!
//! The import rewrites a bare alias to `<Self as Trait>::Name`, and name resolution reports a name
//! the trait does not declare (`E0576`) at the alias the program wrote. The compiler's help respells
//! that alias, which leaves the import's name in place, so the edit does not compile. Once
//! resolution is over the driver's `after_expansion` callback runs [`reshape_use_type_misspellings`],
//! which confirms the import ([`detect::detect_use_type_misspelling`]), drops each suggestion that
//! edits the alias, and adds a `help` naming the rename in the import. The header stays the
//! compiler's, since it already names the missing type and its trait.
//!
//! The buffer is read and written under the output's lock, and the detection, which runs compiler
//! queries, runs with the lock released, as in the name-resolution pass.

use cargo_cgp_error_processing::use_type_misspelling_help;
use rustc_errors::codes::E0576;
use rustc_errors::{CodeSuggestion, Sublevel};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::emitter::buffer::BufEntry;
use crate::emitter::buffer_pass::buffer_pass;
use crate::emitter::edit::{main_message_text, retain_suggestions, subdiag};
use crate::{detect, resolve};

/// Replace the compiler's help on each buffered `E0576` that a misspelled `#[use_type]` import
/// produced. Call once name resolution is over, while a `TyCtxt` is available. A panic in the
/// detection leaves every diagnostic as rustc wrote it.
pub fn reshape_use_type_misspellings(tcx: TyCtxt<'_>) {
    if !resolve::query::links_cgp(tcx) {
        return;
    }
    buffer_pass(
        missing_associated_types,
        |candidates| {
            candidates
                .iter()
                .map(|(_, span, type_name, trait_name, notes)| {
                    detect::detect_use_type_misspelling(tcx, *span, type_name, trait_name, notes)
                })
                .collect::<Vec<_>>()
        },
        |buffer, candidates, misspellings| {
            for ((index, span, ..), misspelling) in candidates.iter().zip(misspellings) {
                let Some(misspelling) = misspelling else {
                    continue;
                };
                let Some(BufEntry::Plain(diag)) = buffer.get_mut(*index) else {
                    continue;
                };
                retain_suggestions(diag, |suggestion| !edits_span(suggestion, *span));
                diag.children.push(subdiag(
                    Sublevel::Help,
                    use_type_misspelling_help(&misspelling),
                ));
            }
        },
    );
}

/// Whether a substitution of `suggestion` edits the text at `span`.
fn edits_span(suggestion: &CodeSuggestion, span: Span) -> bool {
    suggestion
        .substitutions
        .iter()
        .flat_map(|substitution| &substitution.parts)
        .any(|part| part.span.overlaps(span))
}

/// Each buffered `E0576` "cannot find associated type `name` in trait `Trait`", as its buffer
/// index, primary span, type name, trait name, and the spans its notes point at, in buffer order.
fn missing_associated_types(buffer: &[BufEntry]) -> Vec<(usize, Span, String, String, Vec<Span>)> {
    buffer
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let BufEntry::Plain(diag) = entry else {
                return None;
            };
            if diag.code != Some(E0576) {
                return None;
            }
            let (type_name, trait_name) = main_message_text(diag)?
                .strip_prefix("cannot find associated type `")?
                .strip_suffix('`')?
                .split_once("` in trait `")?;
            let notes = diag
                .children
                .iter()
                .flat_map(|child| child.span.primary_spans().iter().copied())
                .collect();
            Some((
                index,
                diag.span.primary_span()?,
                type_name.to_owned(),
                trait_name.to_owned(),
                notes,
            ))
        })
        .collect()
}
