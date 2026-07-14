//! What a resolved check failure's dependency chain bottoms out on.
//!
//! These are the rustc-free leaf types the driver's typed resolver produces and the
//! [wording](super::wording) turns into diagnostic text. They carry only owned `String`s,
//! no compiler handles, so they outlive the inference contexts the driver reads them from
//! and are exercised by plain unit tests.

/// Why a required `HasField` bound is unmet — the distinction that tells a genuinely missing
/// field apart from one some struct actually carries but has not derived. CGP's `HasField`
/// follows `Deref` (a blanket impl forwards to the target), so a field on a `Deref` target
/// resolves when the target derives it; the failure diagnosed here is a field present on some
/// struct that has no `HasField` impl for it. (A field present *with a mismatched type* keeps
/// its `HasField` trait impl and fails only the associated-type projection, so it never reaches
/// this classification — it is a [`Leaf::FieldTypeMismatch`] instead.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldIssue {
    /// No struct in the context's `Deref` chain carries a field of this name: it is genuinely
    /// missing and must be added.
    Missing,
    /// The context struct itself carries a field of this name, yet the `HasField` bound is unmet:
    /// the struct is missing (or has an incomplete) `#[derive(HasField)]` for it.
    Present,
    /// The context does not carry the field directly, but a struct reached through its `Deref`
    /// chain does. Since `HasField` follows `Deref`, the bound would hold if that target derived
    /// the field; the fault is that the target does not derive `HasField`.
    PresentViaDeref {
        /// The `Deref`-reachable struct that carries the field, e.g. `AppFields`.
        target: String,
    },
}

/// What a resolved dependency chain bottoms out on — the actual root cause the tree leads to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leaf {
    /// A `HasField` bound the wiring needs. The emitter renders this as a clean, tree-first
    /// diagnostic of its own, with a lead worded by the [`FieldIssue`] and a derive `help`.
    Field {
        /// The field name, decoded from its `Symbol!`, e.g. `height`.
        name: String,
        /// The struct the `HasField` bound lands on — the type that must carry (or derive) the
        /// field. Usually the checked context, but a nested getter can make it another type.
        owner: String,
        /// Whether the field is genuinely missing, present-but-underived, or behind a `Deref`.
        issue: FieldIssue,
    },
    /// A `HasField` bound whose field is present but carries the wrong type: the trait bound holds
    /// and only the `<owner as HasField<name>>::Value == expected` projection fails. The emitter
    /// words this as a `[CGP-E003]` main message of its own.
    FieldTypeMismatch {
        /// The field name, decoded from its `Symbol!`, e.g. `height`.
        name: String,
        /// The struct that carries the field, e.g. `Rectangle`.
        owner: String,
        /// The type the wiring requires the field to have, taken from the failing projection's
        /// right-hand side, e.g. `f64`.
        expected: String,
        /// The type the field actually has, queried from the struct by `DefId`, e.g. `i32`.
        actual: String,
    },
    /// A component the wiring needs but the context does not delegate at all — the
    /// `DelegateComponent<Marker>` bound has no impl, so no provider is chosen for it. Parallel
    /// to a genuinely missing field, but the fix is to wire the component rather than add a
    /// field. The emitter renders this as a tree-first diagnostic of its own, with a lead naming
    /// the component to wire.
    MissingWiring {
        /// The component marker the context does not wire, e.g. `BarProviderComponent` — the name
        /// the programmer writes on the left of a `delegate_components!` entry to fix it.
        component: String,
        /// The context that must wire the component, e.g. `App`.
        owner: String,
    },
    /// Any other terminal unmet bound — an ordinary trait bound (`f64: Eq`), an unmet abstract
    /// type, and so on. The emitter keeps rustc's own header for these and only replaces the
    /// sub-notes with the dependency tree.
    Bound {
        /// The bound restated as `self: Trait`, e.g. `f64: std::cmp::Eq`, for the note lead and
        /// for de-duplicating a leaf reached by several paths.
        summary: String,
    },
}

impl Leaf {
    /// A stable key that de-duplicates a leaf reached by several dependency paths — the field
    /// name for a field, the bound restatement otherwise.
    pub fn key(&self) -> &str {
        match self {
            Leaf::Field { name, .. } | Leaf::FieldTypeMismatch { name, .. } => name,
            Leaf::MissingWiring { component, .. } => component,
            Leaf::Bound { summary } => summary,
        }
    }
}
