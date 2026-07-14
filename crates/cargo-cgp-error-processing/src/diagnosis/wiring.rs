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
//! The wording names *which keys* collide, in the terms the programmer wrote them (an `@`-path
//! renders in bare `@a.b.*` notation, not wrapped in `Path!(…)`), so the headline is the fix
//! rather than a screen of `DelegateComponent<…>` type spine. Each shape carries its own
//! `[CGP-E0xx]` code, since each rewrites the message into a distinct form with its own fix.

use crate::code::{
    DUPLICATE_REDIRECT, DUPLICATE_WIRING, MULTIPLE_NAMESPACES, OVERLAPPING_WIRING,
    REDIRECT_COLLISION,
};

/// One key of a conflicting `delegate_components!` entry, rendered in the surface form the
/// programmer wrote it. The driver decides which variant a key is by inspecting the impl the
/// conflict lands on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringKey {
    /// A bare component marker, e.g. `GreeterComponent`.
    Component(String),
    /// An `@`-path key, in bare surface notation (`@a.b.*`); a generic tail or loop parameter
    /// collapses to the trailing `.*` wildcard.
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

/// A recognized duplicate-key wiring conflict, in the five shapes the driver tells apart. Every
/// shape carries the `context` it is wired on; the wording (and the `[CGP-E0xx]` code) differs by
/// how the two entries relate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WiringConflict {
    /// Two entries wire the same key directly — the same component or the same `@`-path mapped
    /// twice (`CGP-E004`).
    Duplicate { context: String, key: WiringKey },
    /// Two entries wire *overlapping but distinct* keys, so one cannot claim what the other
    /// already covers — a generic entry over a specific one, or a path that is a prefix of
    /// another (`CGP-E005`).
    Overlap {
        context: String,
        conflicting: WiringKey,
        first: WiringKey,
    },
    /// A context joins more than one namespace, so two blanket forwardings each cover every key
    /// and overlap. `first` and `second` are the two namespace/table trait names (`CGP-E006`).
    MultipleNamespaces {
        context: String,
        first: String,
        second: String,
    },
    /// One entry redirects the key (via `open` or a namespace) while the other sets it directly to
    /// `provider`, so the direct wiring never takes effect; the fix is to wire `provider` under the
    /// redirected `path` instead (`CGP-E007`).
    Redirect {
        context: String,
        key: WiringKey,
        path: String,
        provider: String,
    },
    /// The same key is redirected more than once. `first_path` and `second_path` are the two
    /// redirect targets, equal when both redirects point the same way (`CGP-E008`).
    DuplicateRedirect {
        context: String,
        key: WiringKey,
        first_path: String,
        second_path: String,
    },
}

/// Word a [`WiringConflict`] into the `[CGP-E0xx]` header the emitter puts on the surviving
/// `E0119`, naming the colliding key(s) in the programmer's own terms. The compiler's two
/// carets ("first implementation here" / "conflicting implementation") are kept alongside, so
/// the header says *what* collided and the carets say *where*.
pub fn plan_wiring_conflict(conflict: &WiringConflict) -> String {
    match conflict {
        WiringConflict::Duplicate { context, key } => format!(
            "[{DUPLICATE_WIRING}] duplicate wiring for {} on `{context}`",
            key.noun(),
        ),
        WiringConflict::Overlap {
            context,
            conflicting,
            first,
        } => format!(
            "[{OVERLAPPING_WIRING}] `{context}` cannot wire {} that is already set through {}",
            conflicting.conflicting_phrase(),
            first.source_phrase(),
        ),
        WiringConflict::MultipleNamespaces {
            context,
            first,
            second,
        } => format!(
            "[{MULTIPLE_NAMESPACES}] only one namespace can be used for each target type in \
             `delegate_components!`, but `{context}` uses both `{first}` and `{second}`",
        ),
        WiringConflict::Redirect {
            context, key, path, ..
        } => format!(
            "[{REDIRECT_COLLISION}] {} on `{context}` is redirected to `{path}`",
            key.noun(),
        ),
        WiringConflict::DuplicateRedirect {
            context,
            key,
            first_path,
            second_path,
        } => {
            if first_path == second_path {
                format!(
                    "[{DUPLICATE_REDIRECT}] duplicate redirect for {} on `{context}` \
                     (redirected to `{first_path}`)",
                    key.noun(),
                )
            } else {
                format!(
                    "[{DUPLICATE_REDIRECT}] duplicate redirect for {} on `{context}`: redirected \
                     to both `{first_path}` and `{second_path}`",
                    key.noun(),
                )
            }
        }
    }
}

/// The `help` message accompanying a conflict's header, when it has one. A redirect collision names
/// how to fix it — wire the provider under the redirected key rather than setting the key directly
/// — kept out of the header so the headline stays one short sentence. Every other shape's carets are
/// the fix, so they carry no help.
pub fn wiring_conflict_help(conflict: &WiringConflict) -> Option<String> {
    match conflict {
        WiringConflict::Redirect { path, provider, .. } => Some(format!(
            "wire the provider `{provider}` with the key `{path}`"
        )),
        _ => None,
    }
}
