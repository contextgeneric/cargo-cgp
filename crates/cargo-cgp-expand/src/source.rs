//! Resugaring one expanded source file, end to end.

use crate::options::ExpandOptions;
use crate::resugar::resugar_file;
use crate::resugar::spacing::tighten_sugar_bodies;
use crate::select::select_items;

/// Resugar the CGP constructs in `source`, the compiler's printed expansion, and return it
/// re-printed: narrowed to `options`' item, empty when that matches nothing, and unchanged when
/// `syn` cannot parse it; see `cgp-knowledge-base/cargo-cgp/implementation/expand-command.md`.
pub fn resugar_expanded_source(source: &str, options: &ExpandOptions) -> String {
    let Ok(mut file) = syn::parse_file(source) else {
        return source.to_owned();
    };

    if let Some(item) = &options.item
        && !select_items(&mut file, item)
    {
        return String::new();
    }

    resugar_file(&mut file);
    // The printer spaces a macro body's tokens apart, so the resugared calls are tightened after
    // printing; see `resugar::spacing`.
    tighten_sugar_bodies(&prettyplease::unparse(&file))
}
