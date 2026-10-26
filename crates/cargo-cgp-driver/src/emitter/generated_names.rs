//! Dropping a suggestion that names an identifier a CGP macro generated.
//!
//! A CGP macro declares some names only in its expansion, and spells each as a reserved identifier,
//! wrapped in double underscores (`__Context__`, the lifetime `'__a__`). When the macro cannot name
//! what the user wrote, the compiler reports the gap and may suggest inserting one of those names
//! into the user's source, where it does not resolve. Such a suggestion is wrong whatever the
//! error, so the pass drops it and leaves the diagnostic's other suggestions in place.
//!
//! rustc can offer such a name as one alternative among several (each trait that would bound a type
//! parameter, as one suggestion), so the drop removes the alternative and keeps the others.
//!
//! The emitter applies the drop to each diagnostic of a crate that uses CGP as it arrives. A
//! diagnostic name resolution raises arrives before the emitter can tell, so the driver's
//! `after_expansion` callback applies it again over the diagnostics name resolution buffered, the
//! lifetime errors among them. It reads the buffer through the same lock the name-resolution pass
//! uses, and changes only a crate that links CGP.

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
    if !resolve::links_cgp(tcx) {
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
pub(crate) fn drop_generated_names(diag: &mut DiagInner, source_map: &SourceMap) {
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
