//! The component-marker → trait-names map the rewrite and the typed resolver look names up in.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::rewrite::text::last_segment;

/// The consumer and provider trait names behind one component marker, recovered from the
/// compiler. Keyed in the map by the marker's *full path* (e.g.
/// `my_crate::area::AreaCalculatorComponent`), so two markers that share a name in different
/// modules never collide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentTraitNames {
    /// The consumer trait a context implements (e.g. `CanCalculateArea`).
    pub consumer: String,
    /// The provider trait a provider implements (e.g. `AreaCalculator`).
    pub provider: String,
    /// How many type parameters the component has. CGP groups two or more into a tuple in the
    /// `CanUseComponent`/`IsProviderFor` parameter slot, so this tells that tuple from the single
    /// parameter of a component whose one parameter is a tuple.
    pub params: usize,
}

/// A lazily-built map from a component-marker's full path to the trait names behind it.
///
/// See `cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md` for why it is
/// lazy, keyed by full path, and built by a `fn` pointer.
pub struct ComponentNameMap {
    /// The underlying lazily-initialized map, keyed by each marker's full path. Public so the
    /// driver and tests can also construct one directly, though [`new`](Self::new) is the usual
    /// way.
    pub name_map: LazyLock<HashMap<String, ComponentTraitNames>>,
}

impl ComponentNameMap {
    /// Wrap a map initializer. `init` is a function pointer, so no state is captured here; it
    /// runs lazily on the first lookup.
    pub fn new(init: fn() -> HashMap<String, ComponentTraitNames>) -> Self {
        Self {
            name_map: LazyLock::new(init),
        }
    }

    /// Look up the trait names behind a marker as rustc printed it: a bare name, or a path when
    /// rustc qualified a name that two items share. The printed path is matched against each
    /// full-path key's trailing segments, falling back to the bare name when no key ends in the
    /// whole path (a re-export can print a path the key does not end in). When the keys that match
    /// stand for different traits, the text cannot tell which one it meant, so the lookup finds
    /// nothing and the rewrite leaves the message as rustc wrote it. Forces the lazy build on first
    /// use.
    pub fn get(&self, printed: &str) -> Option<ComponentTraitNames> {
        let suffix = format!("::{printed}");
        let by_path: Vec<&ComponentTraitNames> = self
            .name_map
            .iter()
            .filter(|(path, _)| *path == printed || path.ends_with(&suffix))
            .map(|(_, entry)| entry)
            .collect();
        let candidates = if by_path.is_empty() {
            let name = last_segment(printed);
            self.name_map
                .iter()
                .filter(|(path, _)| last_segment(path) == name)
                .map(|(_, entry)| entry)
                .collect()
        } else {
            by_path
        };
        let (first, rest) = candidates.split_first()?;
        rest.iter()
            .all(|entry| entry == first)
            .then(|| (*first).clone())
    }

    /// Whether some component's consumer trait is named `name`, as rustc printed it (a bare name,
    /// or a path whose last segment is the name). Forces the lazy build on first use.
    pub fn is_consumer(&self, name: &str) -> bool {
        let name = last_segment(name);
        self.name_map.values().any(|entry| entry.consumer == name)
    }
}
