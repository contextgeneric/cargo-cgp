//! Resugaring one expanded source file, end to end.

use crate::options::ExpandOptions;
use crate::resugar::resugar_file;
use crate::resugar::spacing::tighten_sugar_bodies;

/// Resugar the CGP constructs in `source` — the text the compiler's pretty-printer produced for
/// an expanded crate — and return it re-printed.
///
/// When `source` does not parse, it is returned **unchanged** rather than reported: expansion can
/// produce shapes `syn` does not accept, and degrading to exactly what the compiler printed is far
/// better than failing the command. (`cargo-expand` makes the same call, with an extra `rustfmt`
/// rung in its ladder that this crate does not need.)
pub fn resugar_expanded_source(source: &str, options: &ExpandOptions) -> String {
    let Ok(mut file) = syn::parse_file(source) else {
        return source.to_owned();
    };

    resugar_file(&mut file, options);
    // The printer spaces a macro body's tokens apart, so the resugared calls are tightened after
    // printing; see `resugar::spacing`.
    tighten_sugar_bodies(&prettyplease::unparse(&file))
}
