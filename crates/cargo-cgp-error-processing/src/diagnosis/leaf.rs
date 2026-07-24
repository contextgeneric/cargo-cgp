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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// Several fields of one struct, each present but with no `HasField` implementation — the
    /// coalesced form of two or more [`Leaf::Field`] causes whose issue is
    /// [`FieldIssue::Present`] on the same owner. `#[derive(HasField)]` derives an impl for
    /// *every* field, so several underived fields on one struct are one mistake with one fix
    /// (the missing derive), and reporting them as separate root causes would overstate the
    /// work. Built by [`coalesce_underived_fields`](super::coalesce_underived_fields), never by
    /// the driver's classifier directly; the merged cause's tree still branches to the
    /// per-field leaves, so each field stays visible in the chain.
    UnderivedFields {
        /// The field names with no `HasField` impl, in first-seen order, e.g.
        /// `["height", "width"]`.
        names: Vec<String>,
        /// The struct that carries the fields but not the derive, e.g. `Rectangle`.
        owner: String,
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
    /// A non-context delegation table that has no entry for a key it is asked to resolve. The owner
    /// is a *provider* that delegates — an [aggregate provider](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/aggregate-providers.md)
    /// missing a component wiring, or a
    /// [`UseDelegate`](https://github.com/contextgeneric/cgp/blob/main/docs/reference/providers/use_delegate.md) / `UseInputDelegate`
    /// dispatch table missing a branch for the type it dispatches on (a `Code` fragment or an `Input`
    /// value's type). Parallel to [`Leaf::MissingWiring`], but the owner is a provider table rather
    /// than the context: the fix is to add the entry to *that provider's* table (or feed the stage a
    /// type the table already covers), not to wire a component on the context.
    MissingDispatchEntry {
        /// The key with no entry — a component marker (`BarProviderComponent`) for an aggregate
        /// provider, or a dispatched-on type (`GenericArray<u8, U32>`) for a `UseDelegate` /
        /// `UseInputDelegate` table.
        key: String,
        /// The provider table that lacks the entry, e.g. `CommonProvider` or
        /// `ToTokioAsyncReadHandlers`.
        table: String,
    },
    /// A type wired where a *provider* was expected that does not implement the provider trait at
    /// all. A higher-order provider's inner slot (or any wiring position that expects a provider) is
    /// filled with a type that has no impl of the provider trait — the `money-transfer-api` mistake
    /// of wiring `UseBasicAuth<QueryBalanceRequest>`, where the *request* type `QueryBalanceRequest`
    /// sits where an `ApiHandler` provider belongs. Distinct from [`Leaf::MissingDispatchEntry`]: the
    /// owner is not a delegation table missing one key, it is simply not a provider — so the fix is to
    /// use an actual provider (e.g. wrap it in the endpoint handler), not to add a wiring entry.
    NotAProvider {
        /// The type wired where a provider was expected, e.g. `QueryBalanceRequest`.
        provider: String,
        /// The provider trait it fails to implement, e.g. `ApiHandler`.
        provider_trait: String,
    },
    /// A namespace redirect whose target path has no delegate entry. A `RedirectLookup<Ctx, Path>`
    /// provider resolves a component by forwarding the lookup to `Path` inside `Ctx`'s wiring
    /// (through the namespace lookup trait every joined namespace supplies), but nothing — no direct
    /// entry, no namespace default, no `#[default_impl]` — terminates that path with a provider, so
    /// the redirect resolves to nothing. Parallel to [`Leaf::MissingWiring`], but keyed by a redirect
    /// *path* rather than a bare component marker: the fix is to add a wiring entry for the path (on
    /// the context, or in the namespace it joins).
    MissingRedirectWiring {
        /// The redirect path with no terminating entry, e.g.
        /// `Path!(@app.finance.types.QuantityTypeProviderComponent)` (resugared from its `PathCons`
        /// spine when the emitter post-processes the note).
        path: String,
        /// The context the lookup is forwarded inside, e.g. `MockApp` — the table that carries no
        /// delegate entry for the path.
        context: String,
    },
    /// An associated type the wiring projects through that carries a different type from the one a
    /// provider requires. Like [`Leaf::FieldTypeMismatch`], the trait bound itself holds and only the
    /// `<owner as Trait>::Assoc == expected` projection fails; unlike it, the projection is *not*
    /// `HasField`'s `Value`. The archetype is a CGP
    /// [abstract type](https://github.com/contextgeneric/cgp/blob/main/docs/concepts/abstract-types.md):
    /// the context binds `HasErrorType::Error` to one concrete type by wiring
    /// `ErrorTypeProviderComponent` to `UseType<T>`, while a provider pins it to another with the
    /// `#[use_type(HasErrorType.{Error = AppError})]` equality form. The emitter words this as a
    /// `[CGP-E017]` main message of its own.
    AssocTypeMismatch {
        /// The associated type's name, e.g. `Error`.
        assoc: String,
        /// The trait that declares it, e.g. `HasErrorType`.
        trait_name: String,
        /// The type the projection is taken on — the context for an abstract type, e.g. `App`.
        owner: String,
        /// The type the wiring requires, taken from the failing projection's right-hand side, e.g.
        /// `AppError`.
        expected: String,
        /// The type the owner actually supplies, read by normalizing the projection, e.g. `String`.
        actual: String,
        /// The component marker a context wires to choose this type (`ErrorTypeProviderComponent`),
        /// present only when the trait is a CGP abstract-type component — a consumer trait with one
        /// associated type whose provider carries the `UseType` blanket `#[cgp_type]` generates. It
        /// selects the `abstract type` wording over the plain `associated type` one, and supplies the
        /// wiring `help`; `None` for an ordinary trait's associated type, which has no such fix.
        component: Option<String>,
    },
    /// Any other terminal unmet bound — an ordinary trait bound such as `f64: Eq`, or a capability
    /// bound the walk cannot descend further. The emitter keeps rustc's own header for these and
    /// only replaces the sub-notes with the dependency tree.
    Bound {
        /// The bound restated as `self: Trait`, e.g. `f64: std::cmp::Eq`, for the note lead and
        /// for de-duplicating a leaf reached by several paths.
        summary: String,
    },
}

impl Leaf {
    /// A stable key that de-duplicates a leaf reached by several dependency paths — the field name
    /// for a field, the associated-type name for a projection mismatch, the bound restatement
    /// otherwise. The key names the *thing to fix* rather than the whole leaf, so two leaves that
    /// name the same thing on different owners merge into one cause; the merged cause keeps every
    /// path, so the tree still branches to each leaf and nothing is lost but the heading's
    /// precision.
    pub fn key(&self) -> &str {
        match self {
            Leaf::Field { name, .. } | Leaf::FieldTypeMismatch { name, .. } => name,
            Leaf::UnderivedFields { owner, .. } => owner,
            Leaf::MissingWiring { component, .. } => component,
            Leaf::MissingDispatchEntry { key, .. } => key,
            Leaf::NotAProvider { provider, .. } => provider,
            Leaf::MissingRedirectWiring { path, .. } => path,
            Leaf::AssocTypeMismatch { assoc, .. } => assoc,
            Leaf::Bound { summary } => summary,
        }
    }
}
