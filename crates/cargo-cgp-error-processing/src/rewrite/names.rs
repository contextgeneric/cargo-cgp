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
}

/// A lazily-built map from a component-marker's full path to the trait names behind it.
///
/// Building the map is expensive — the driver walks the whole trait graph through a
/// `TyCtxt` — so it is wrapped in a [`LazyLock`]: the initializer runs at most once, on the
/// first lookup, and *not at all* if no message is ever rewritten. That is what lets the
/// emitter drop a separate "does this diagnostic mention CGP?" pre-filter: the rewrite
/// functions look a marker up only after a message parses as a wiring form, so an ordinary
/// diagnostic never forces the map.
///
/// The map has two lookup paths for its two callers. The driver's typed resolver holds a
/// marker's `DefId` and looks it up by [`get_by_path`](Self::get_by_path) — an exact full-path
/// match that can never confuse two same-named markers from different modules. The text rewrite
/// has only the marker *name* rustc printed (rarely a full path), so it uses [`get`](Self::get),
/// which matches a key by its last path segment; that is inherently ambiguous when two markers
/// share a name, a residual the text form cannot resolve and the typed path avoids.
///
/// The initializer is a plain `fn` pointer, not a closure, so this type captures no compiler
/// state and can live in this rustc-free crate. The driver supplies a `fn` that reads the
/// `TyCtxt` from thread-local scope and builds the map (valid because a wiring message is
/// emitted during trait solving, when a `TyCtxt` is in scope); the tests supply a `fn` that
/// returns a fixed map.
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

    /// Look up the trait names behind a marker by its full path — the exact, collision-free
    /// lookup the driver's typed resolver uses. Forces the lazy build on first use.
    pub fn get_by_path(&self, path: &str) -> Option<ComponentTraitNames> {
        self.name_map.get(path).cloned()
    }

    /// Look up the trait names behind a marker by its bare name, matching a full-path key by its
    /// last segment — the lookup the text rewrite uses, since rustc prints the marker unqualified.
    /// When two markers share a name (in different modules) the match is arbitrary; the text form
    /// cannot tell them apart, whereas the typed resolver keys on the full path via
    /// [`get_by_path`](Self::get_by_path). Forces the lazy build on first use.
    pub fn get(&self, name: &str) -> Option<ComponentTraitNames> {
        self.name_map
            .iter()
            .find(|(path, _)| last_segment(path) == name)
            .map(|(_, entry)| entry.clone())
    }
}
