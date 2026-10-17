//! Dropping a suggestion that names an identifier a CGP macro generated.
//!
//! A CGP macro declares some names only in its expansion, and spells each as a reserved identifier,
//! wrapped in double underscores (`__Context__`, the lifetime `'__a__`). When the macro cannot name
//! what the user wrote, the compiler reports the gap and may suggest inserting one of those names
//! into the user's source, where it does not resolve. Such a suggestion is wrong whatever the
//! error, so the pass drops it and leaves the diagnostic's other suggestions in place.
//!
//! The pass runs from the driver's `after_expansion` callback, over the diagnostics name resolution
//! buffered, since the lifetime errors arrive then. It reads the buffer through the same lock the
//! name-resolution pass uses, and changes only a crate that links CGP.

use cargo_cgp_error_processing::reserved_names;
use rustc_errors::CodeSuggestion;
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::SourceMap;

use crate::emitter::buffer::BufEntry;
use crate::emitter::edit::retain_suggestions;
use crate::emitter::output::with_registered_buffer;
use crate::resolve;

/// Drop, from every buffered diagnostic, each suggestion that inserts a reserved name the source
/// file it edits never writes. Call once name resolution is over, while a `TyCtxt` is available.
pub fn drop_generated_name_suggestions(tcx: TyCtxt<'_>) {
    if !resolve::links_cgp(tcx) {
        return;
    }
    let source_map = tcx.sess.source_map();
    with_registered_buffer(|buffer| {
        for entry in buffer {
            if let BufEntry::Plain(diag) = entry {
                retain_suggestions(diag, |suggestion| {
                    !inserts_generated_name(source_map, suggestion)
                });
            }
        }
    });
}

/// Whether a substitution of `suggestion` writes a reserved name that the file it edits does not
/// contain, so the name can only come from a macro's expansion.
fn inserts_generated_name(source_map: &SourceMap, suggestion: &CodeSuggestion) -> bool {
    suggestion
        .substitutions
        .iter()
        .flat_map(|substitution| &substitution.parts)
        .any(|part| {
            let written = source_map.lookup_source_file(part.span.lo()).src.clone();
            reserved_names(&part.snippet).into_iter().any(|name| {
                !written
                    .as_deref()
                    .is_some_and(|source| source.contains(name))
            })
        })
}
