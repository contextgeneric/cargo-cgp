//! The rustc-free model and wording for a duplicate-key wiring conflict.
//!
//! A duplicate key in `delegate_components!` makes the expansion emit two overlapping
//! `DelegateComponent` impls, which the compiler rejects with a coherence error (`E0119`).
//! The driver recognizes the pair, drops the redundant `IsProviderFor` half, and hands the
//! surviving `DelegateComponent` half's conflict — recovered by querying the trait solver — to
//! [`plan_wiring_conflict`], which words it into the replacement header. Keeping the model and
//! wording here, in owned `String` form, is what makes them unit-testable without a compiler;
//! the driver's `resolve::conflict` module fills the model in from the live `TyCtxt`.
//!
//! The wording names *which keys* collide, in the terms the programmer wrote them, so the
//! headline is the fix rather than a screen of `DelegateComponent<…>` type spine.

use crate::code::CONFLICTING_WIRING;

/// One key of a conflicting `delegate_components!` entry, rendered in the surface form the
/// programmer wrote it. The driver decides which variant a key is by inspecting the impl the
/// conflict lands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringKey {
    /// A bare component marker, e.g. `GreeterComponent`.
    Component(String),
    /// An `@`-path key, already rendered as its `Path!(@a.b.*)` surface form (a generic tail
    /// or loop parameter collapses to the trailing `.*` wildcard).
    Path(String),
    /// A blanket forwarding that covers *every* key, tagged by the namespace or table trait
    /// that keys it (e.g. `DefaultNamespace`) — the shape a `namespace …;` join or a bare-key
    /// `for` loop lowers to.
    Blanket(String),
}

impl WiringKey {
    /// The key as the subject of a "duplicate wiring for …" sentence.
    fn noun(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Path(path) => format!("`{path}`"),
            WiringKey::Blanket(trait_name) => format!("every key forwarded through `{trait_name}`"),
        }
    }

    /// The key as the object of "cannot wire …" — the entry that introduces the overlap.
    fn conflicting_phrase(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Path(path) => format!("`{path}`"),
            WiringKey::Blanket(trait_name) => format!("a key through `{trait_name}`"),
        }
    }

    /// The key as the source in "already set through …" — the entry that already claims it.
    fn source_phrase(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Path(path) => format!("`{path}`"),
            WiringKey::Blanket(trait_name) => format!("`{trait_name}`"),
        }
    }
}

/// A recognized duplicate-key wiring conflict, in the four shapes the driver tells apart. Every
/// shape carries the `context` it is wired on; the wording differs by how the two entries relate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringConflict {
    /// Two entries wire the same key directly — the same component or the same `@`-path mapped
    /// twice.
    Duplicate { context: String, key: WiringKey },
    /// Two entries wire *overlapping but distinct* keys, so one cannot claim what the other
    /// already covers — a generic entry over a specific one, two namespace forwardings, or a
    /// path that is a prefix of another.
    Overlap {
        context: String,
        conflicting: WiringKey,
        first: WiringKey,
    },
    /// One entry redirects the key (via `open` or a namespace) while the other sets it, so the
    /// direct wiring never takes effect; the fix is to set the redirected `path` instead.
    Redirect {
        context: String,
        key: WiringKey,
        path: String,
    },
    /// The same key is redirected more than once — two `open`s or redirects of one key.
    DuplicateRedirect {
        context: String,
        key: WiringKey,
        path: String,
    },
}

/// Word a [`WiringConflict`] into the `[CGP-E004]` header the emitter puts on the surviving
/// `E0119`, naming the colliding key(s) in the programmer's own terms. The compiler's two
/// carets ("first implementation here" / "conflicting implementation") are kept alongside, so
/// the header says *what* collided and the carets say *where*.
pub fn plan_wiring_conflict(conflict: &WiringConflict) -> String {
    let code = CONFLICTING_WIRING;
    match conflict {
        WiringConflict::Duplicate { context, key } => {
            format!(
                "[{code}] duplicate wiring for {} on `{context}`",
                key.noun()
            )
        }
        WiringConflict::Overlap {
            context,
            conflicting,
            first,
        } => format!(
            "[{code}] `{context}` cannot wire {} that is already set through {}",
            conflicting.conflicting_phrase(),
            first.source_phrase(),
        ),
        WiringConflict::Redirect { context, key, path } => format!(
            "[{code}] {} on `{context}` is redirected to `{path}`; set the redirected key \
             instead of wiring it directly",
            key.noun(),
        ),
        WiringConflict::DuplicateRedirect { context, key, path } => format!(
            "[{code}] duplicate redirect for {} on `{context}` (redirected to `{path}`)",
            key.noun(),
        ),
    }
}
