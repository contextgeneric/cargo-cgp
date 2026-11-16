//! Running the resugaring passes over a whole file, in the order they must run in.

use syn::File;
use syn::visit_mut::VisitMut;

use crate::resugar::list::Lists;
use crate::resugar::path::Paths;
use crate::resugar::strip::StripPrelude;
use crate::resugar::symbol::Symbols;

/// Resugar every CGP construct in `file`, in place, after dropping the `cgp::macro_prelude::`
/// qualifier. The passes run as separate whole-tree visits in a fixed order, because each reads the
/// last's output and `Nil` ends three lists; see
/// `cgp-knowledge-base/cargo-cgp/implementation/resugaring.md`.
pub fn resugar_file(file: &mut File) {
    StripPrelude.visit_file_mut(file);
    Symbols.visit_file_mut(file);
    Paths.visit_file_mut(file);
    Lists.visit_file_mut(file);
}
