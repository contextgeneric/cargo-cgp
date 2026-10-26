//! Which qualified type names are ambiguous, so a path strip leaves them qualified.

use std::collections::{HashMap, HashSet};

/// Which qualified paths a strip must leave qualified, because the bare name would read as another
/// item (`std::io::Error` beside `backend::Error`). With trimmed paths rustc prints a name qualified
/// only when the name is not unique or the crate glob-imports it, so a qualified name the crate
/// does not glob-import is ambiguous. A glob-imported name is qualified whether or not it collides,
/// so for each one the set records the paths of the items the crate's globs bring in under it, in
/// each form rustc prints them: a printed path among them is that item and reads the same bare, and
/// any other path names a different item. The default knows nothing and keeps no qualifier.
#[derive(Clone, Debug, Default)]
pub struct AmbiguousNames {
    glob_imported: HashMap<String, HashSet<String>>,
    known: bool,
}

impl AmbiguousNames {
    /// The set implied by the names the crate glob-imports, each with the full paths of the items
    /// its globs bring in (empty when they could not be read, which keeps no qualifier on the
    /// name), and `root_names`, the first segments of a path rustc prints from the crate root or
    /// from another crate: the crate root's children and the crates' names.
    pub fn new(
        glob_imported: impl IntoIterator<Item = (String, Vec<String>)>,
        root_names: impl IntoIterator<Item = String>,
    ) -> Self {
        let root_names: HashSet<String> = root_names.into_iter().collect();
        let mut map: HashMap<String, HashSet<String>> = HashMap::new();
        for (name, paths) in glob_imported {
            let printed = map.entry(name).or_default();
            for path in paths {
                printed.extend(printed_tails(&path, &root_names));
                printed.insert(path);
            }
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

/// The shorter paths rustc prints for the item at the full `path`: it prints a parent module by its
/// name alone when no other item shares that name, so the item can print from any enclosing module
/// on (`add::EvalAdd` for `providers::eval::add::EvalAdd`). A tail starting at one of `root_names`
/// is left out, since a printed path starting there is one from the crate root or another crate,
/// which names a different item.
fn printed_tails(path: &str, root_names: &HashSet<String>) -> Vec<String> {
    let segments: Vec<&str> = path.split("::").collect();
    (1..segments.len().saturating_sub(1))
        .filter(|&start| !root_names.contains(segments[start]))
        .map(|start| segments[start..].join("::"))
        .collect()
}
