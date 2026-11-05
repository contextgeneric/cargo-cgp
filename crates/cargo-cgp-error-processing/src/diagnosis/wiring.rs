//! The rustc-free model and wording for a duplicate-key wiring conflict.
//!
//! A duplicate key in `delegate_components!` makes the expansion emit two overlapping
//! `DelegateComponent` impls, which the compiler rejects with a coherence error (`E0119`).
//! The driver recognizes the pair, drops the redundant `IsProviderFor` half, and hands the
//! surviving `DelegateComponent` half's conflict, recovered by querying the trait solver, to
//! [`plan_wiring_conflict`], which words it into the replacement header. Keeping the model and
//! wording here, in owned `String` form, makes them unit-testable without a compiler;
//! the driver's `detect::conflict` module fills the model in from the live `TyCtxt`.
//!
//! The wording names *which keys* collide, in the terms the programmer wrote them (an `@`-path
//! renders in bare `@a.b` notation, not wrapped in `Path!(…)`), so the headline is the fix
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
    /// An ordinary type used as a wiring key, e.g. the `String` of a per-type default or the
    /// `u64` of a dispatch table: a key that names no component.
    Type(String),
    /// An `@`-path key, in bare surface notation as its entry wrote it (`@a.b`,
    /// `@ComputerComponent.Code.u64`): a generic segment keeps its parameter name, and the
    /// wildcard tail the macros append is left out.
    Path(String),
    /// A blanket forwarding that covers *every* key, tagged by the namespace or table trait
    /// that keys it (e.g. `DefaultNamespace`): the shape a `namespace …;` join or a bare-key
    /// `for` loop lowers to.
    Blanket(String),
}

impl WiringKey {
    /// The key as the object of "wires … twice" or "redirects … twice".
    fn noun(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Type(name) => format!("type `{name}`"),
            WiringKey::Path(path) => format!("`{path}`"),
            WiringKey::Blanket(trait_name) => format!("every key forwarded through `{trait_name}`"),
        }
    }

    /// The key as the object of "wires …, which … already covers": the entry that introduces the
    /// overlap.
    fn conflicting_phrase(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Type(name) => format!("type `{name}`"),
            WiringKey::Path(path) => format!("`{path}`"),
            WiringKey::Blanket(trait_name) => format!("a key through `{trait_name}`"),
        }
    }

    /// The key as the subject of "which … already covers": the entry that already claims it.
    fn source_phrase(&self) -> String {
        match self {
            WiringKey::Component(name) => format!("component `{name}`"),
            WiringKey::Type(name) => format!("type `{name}`"),
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
    /// Two entries wire the same key directly: the same component or the same `@`-path mapped
    /// twice (`CGP-E010`).
    Duplicate {
        /// The context or namespace the entries are wired on.
        context: String,
        /// The key both entries wire.
        key: WiringKey,
    },
    /// Two entries wire *overlapping but distinct* keys, so one cannot claim what the other
    /// already covers: a generic entry over a specific one, or a path that is a prefix of
    /// another (`CGP-E011`). `first` is the key that covers `conflicting`, which may be the key the
    /// two entries share when neither covers the other.
    Overlap {
        /// The context or namespace the entries are wired on.
        context: String,
        /// The key that collides.
        conflicting: WiringKey,
        /// The key that covers it, or the key the two entries share.
        first: WiringKey,
    },
    /// A context joins more than one namespace, so two blanket forwardings each cover every key
    /// and overlap. `first` and `second` are the two namespace/table trait names (`CGP-E012`).
    MultipleNamespaces {
        /// The context that joins the namespaces.
        context: String,
        /// The first namespace or table trait.
        first: String,
        /// The second namespace or table trait.
        second: String,
    },
    /// One entry redirects the key (via `open` or a namespace) while the other sets it directly to
    /// `provider`, so the direct wiring never takes effect; the fix is to wire `provider` under the
    /// redirected `path` instead (`CGP-E013`). `provider` is `None` when the direct entry names no
    /// provider a reader wrote. `params` are the component's type parameters, which the redirect
    /// appends to `path` when it looks the component up, so a generic component is wired one key
    /// per parameter value rather than at `path` itself.
    Redirect {
        /// The context or namespace the entries are wired on.
        context: String,
        /// The key wired directly.
        key: WiringKey,
        /// The path the key is redirected to.
        path: String,
        /// The provider the direct entry names, when a reader wrote one.
        provider: Option<String>,
        /// The component's type parameters, which the redirect appends to the path.
        params: Vec<String>,
    },
    /// The same key is redirected more than once. `first_path` and `second_path` are the two
    /// redirect targets, equal when both redirects point the same way (`CGP-E014`).
    DuplicateRedirect {
        /// The context or namespace the redirects are on.
        context: String,
        /// The key redirected twice.
        key: WiringKey,
        /// The first redirect's target.
        first_path: String,
        /// The second redirect's target.
        second_path: String,
    },
}

/// Word a [`WiringConflict`] into the `[CGP-E0xx]` header the emitter puts on the surviving
/// `E0119`, naming the colliding key(s) in the programmer's own terms. Every shape opens with
/// `conflicting wiring:`, so a reader meets the class before the detail, then states the subject,
/// the key, and how the entries collide. The compiler's two carets ("first implementation here" /
/// "conflicting implementation") are kept alongside, so the header says *what* collided and the
/// carets say *where*.
pub fn plan_wiring_conflict(conflict: &WiringConflict) -> String {
    match conflict {
        WiringConflict::Duplicate { context, key } => format!(
            "[{DUPLICATE_WIRING}] conflicting wiring: `{context}` wires {} twice",
            key.noun(),
        ),
        WiringConflict::Overlap {
            context,
            conflicting,
            first,
        } => format!(
            "[{OVERLAPPING_WIRING}] conflicting wiring: `{context}` wires {}, which {} already covers",
            conflicting.conflicting_phrase(),
            first.source_phrase(),
        ),
        WiringConflict::MultipleNamespaces {
            context,
            first,
            second,
        } => format!(
            "[{MULTIPLE_NAMESPACES}] conflicting wiring: `{context}` joins both `{first}` and \
             `{second}`, but a target type in `delegate_components!` can join only one namespace",
        ),
        WiringConflict::Redirect {
            context, key, path, ..
        } => format!(
            "[{REDIRECT_COLLISION}] conflicting wiring: `{context}` wires {} directly, but it is \
             redirected to `{path}`",
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
                    "[{DUPLICATE_REDIRECT}] conflicting wiring: `{context}` redirects {} twice, \
                     both times to `{first_path}`",
                    key.noun(),
                )
            } else {
                format!(
                    "[{DUPLICATE_REDIRECT}] conflicting wiring: `{context}` redirects {} twice, to \
                     `{first_path}` and to `{second_path}`",
                    key.noun(),
                )
            }
        }
    }
}

/// The `help` message accompanying a conflict's header, when it has one. A redirect collision names
/// how to fix it: wire the direct entry's provider under the redirected key rather than setting the
/// key directly, kept out of the header so the headline stays one short sentence. A redirect looks
/// a component up with its type parameters appended to the path, so for a generic component the key
/// takes them as a generic segment each, `@AreaCalculatorComponent.<Shape> Shape`, which serves
/// every parameter value the direct entry served. The help is omitted when the direct entry names
/// no provider a reader wrote. Every other shape's carets are the fix, so they carry no help.
pub fn wiring_conflict_help(conflict: &WiringConflict) -> Option<String> {
    let WiringConflict::Redirect {
        path,
        provider,
        params,
        ..
    } = conflict
    else {
        return None;
    };
    let provider = provider.as_ref()?;
    let key = if params.is_empty() {
        path.clone()
    } else {
        format!("{path}.<{}> {}", params.join(", "), params.join("."))
    };
    Some(format!(
        "wire the provider `{provider}` with the key `{key}`"
    ))
}
