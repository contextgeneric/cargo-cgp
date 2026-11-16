//! Dropping a suggestion that names an identifier a CGP macro generated.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md`.

use cargo_cgp_error_processing::reserved_names;
use rustc_errors::{DiagInner, Substitution};
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::SourceMap;

use crate::emitter::buffer::BufEntry;
use crate::emitter::edit::retain_substitutions;
use crate::emitter::output::with_registered_buffer;
use crate::resolve;

/// Drop, from every buffered diagnostic, each suggested substitution that inserts a reserved name
/// the source file it edits never writes. Call once name resolution is over, while a `TyCtxt` is
/// available.
pub fn drop_generated_name_suggestions(tcx: TyCtxt<'_>) {
    if !resolve::query::links_cgp(tcx) {
        return;
    }
    let source_map = tcx.sess.source_map();
    with_registered_buffer(|buffer| {
        for entry in buffer {
            if let BufEntry::Plain(diag) = entry {
                drop_generated_names(diag, source_map);
            }
        }
    });
}

/// Drop each substitution of `diag`'s suggestions that inserts a reserved name the file it edits
/// does not contain, and a suggestion left with none. Call it only for a crate that uses CGP.
pub fn drop_generated_names(diag: &mut DiagInner, source_map: &SourceMap) {
    retain_substitutions(diag, |substitution| {
        !inserts_generated_name(source_map, substitution)
    });
}

/// Whether `substitution` writes a reserved name that the file it edits does not contain, so the
/// name can only come from a macro's expansion.
fn inserts_generated_name(source_map: &SourceMap, substitution: &Substitution) -> bool {
    substitution.parts.iter().any(|part| {
        let written = source_map.lookup_source_file(part.span.lo()).src.clone();
        reserved_names(&part.snippet).into_iter().any(|name| {
            !written
                .as_deref()
                .is_some_and(|source| source.contains(name))
        })
    })
}
