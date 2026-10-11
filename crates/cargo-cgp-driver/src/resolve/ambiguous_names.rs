//! Which qualified type names are ambiguous, for the path strip to leave qualified.

use cargo_cgp_error_processing::AmbiguousNames;
use rustc_middle::ty::TyCtxt;

/// The names the crate glob-imports (the resolver's `glob_map`), from which a qualified name that is
/// not one is ambiguous: with trimmed paths rustc qualifies a name only when it is not unique or
/// glob-imported (`trimmed_def_paths` in `rustc_middle/src/ty/print/pretty.rs`). Reading the map
/// computes nothing under the emitter. Forcing `trimmed_def_paths` itself would, and it takes the
/// diagnostic context's lock, which the emitter holds.
///
/// `None` until name resolution has finished, which the frozen crate store marks (see
/// [`crate_uses_cgp`](super::crate_uses_cgp)).
pub fn ambiguous_type_names(tcx: TyCtxt<'_>) -> Option<AmbiguousNames> {
    if !tcx.untracked().cstore.is_frozen() {
        return None;
    }
    Some(AmbiguousNames::new(
        tcx.resolutions(())
            .glob_map
            .values()
            .flatten()
            .map(|name| name.to_string()),
    ))
}
