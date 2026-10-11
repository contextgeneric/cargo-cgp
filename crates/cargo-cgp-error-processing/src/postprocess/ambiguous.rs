//! Which qualified type names are ambiguous, so a path strip leaves them qualified.

use std::collections::{HashMap, HashSet};

/// Which qualified paths a strip must leave qualified, because the bare name would read as another
/// item (`std::io::Error` beside `backend::Error`). With trimmed paths rustc prints a name qualified
/// only when the name is not unique or the crate glob-imports it, so a qualified name the crate
/// does not glob-import is ambiguous. A glob-imported name is qualified whether or not it collides,
/// so for each one the set records the paths of the items the crate's globs bring in under it: a
/// printed path among them is that item and reads the same bare, and any other path names a
/// different item. The default knows nothing and keeps no qualifier.
#[derive(Clone, Debug, Default)]
pub struct AmbiguousNames {
    glob_imported: HashMap<String, HashSet<String>>,
    known: bool,
}

impl AmbiguousNames {
    /// The set implied by the names the crate glob-imports, each with the paths of the items its
    /// globs bring in (empty when they could not be read, which keeps no qualifier on the name).
    pub fn new(glob_imported: impl IntoIterator<Item = (String, Vec<String>)>) -> Self {
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        for (name, paths) in glob_imported {
            map.entry(name).or_default().extend(paths);
        }
        Self {
            glob_imported: map,
            known: true,
        }
    }

    /// Whether the qualified `path` rustc printed, whose last segment is `name`, must stay qualified
    /// to name its item: the set is known, and either the crate does not glob-import `name` or its
    /// globs bring in a different item under it.
    pub fn keeps_qualifier(&self, path: &str, name: &str) -> bool {
        if !self.known {
            return false;
        }
        match self.glob_imported.get(name) {
            None => true,
            Some(paths) => !paths.is_empty() && !paths.contains(path),
        }
    }
}
