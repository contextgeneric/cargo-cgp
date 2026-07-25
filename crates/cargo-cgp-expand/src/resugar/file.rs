//! Running the resugaring passes over a whole file, in the order they must run in.

use syn::File;
use syn::visit_mut::VisitMut;

use crate::options::ExpandOptions;
use crate::resugar::list::Lists;
use crate::resugar::path::Paths;
use crate::resugar::strip::StripPrelude;
use crate::resugar::symbol::Symbols;

/// Resugar every CGP construct in `file`, in place.
///
/// The passes are **separate whole-tree visits in a fixed order**, and that is load-bearing rather
/// than stylistic. `Symbol!` runs first because the two later passes read the `Symbol!("…")` calls
/// it produces — as a path segment, and as a `Field`'s name tag. `Path!` runs before the lists
/// because `Nil` terminates a path *and* an empty product, so the path pass must consume its own
/// terminator first.
///
/// Folding them into one visitor is a bug, not an optimization: a visitor recurses innermost-first,
/// so it would rewrite a `Symbol`'s terminating `Nil` into `Product![]` before ever examining the
/// enclosing `Symbol`, which then no longer matches — and every field name silently stays raw. See
/// `cgp-knowledge-base/cargo-cgp/implementation/resugaring.md`.
pub fn resugar_file(file: &mut File, options: &ExpandOptions) {
    if options.strip_cgp_prefixes {
        StripPrelude.visit_file_mut(file);
    }
    Symbols.visit_file_mut(file);
    Paths.visit_file_mut(file);
    Lists.visit_file_mut(file);
}
