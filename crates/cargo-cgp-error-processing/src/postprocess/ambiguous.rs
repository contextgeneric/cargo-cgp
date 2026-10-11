//! Which qualified type names are ambiguous, so a path strip leaves them qualified.

use std::collections::HashSet;

/// Which type names name more than one item in the build, so a path ending in one keeps its
/// qualifier (`std::io::Error` beside `backend::Error`). With trimmed paths rustc prints a name
/// qualified only when the name is not unique or the crate glob-imports it, so a qualified name the
/// crate does not glob-import is ambiguous. The set holds the glob-imported names; the default knows
/// none and finds no name ambiguous, which strips every qualifier.
#[derive(Clone, Debug, Default)]
pub struct AmbiguousNames {
    glob_imported: HashSet<String>,
    known: bool,
}

impl AmbiguousNames {
    /// The set implied by the names the crate `glob_imported`.
    pub fn new(glob_imported: impl IntoIterator<Item = String>) -> Self {
        Self {
            glob_imported: glob_imported.into_iter().collect(),
            known: true,
        }
    }

    /// Whether a name rustc printed qualified names more than one item: the set is known and the
    /// crate does not glob-import the name.
    pub fn contains(&self, name: &str) -> bool {
        self.known && !self.glob_imported.contains(name)
    }
}
