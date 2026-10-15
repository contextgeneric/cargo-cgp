//! What a resolved check failure's dependency chain bottoms out on.
//!
//! These are the rustc-free leaf types the driver's typed resolver produces and the
//! [wording](super::wording) turns into diagnostic text. They carry only owned `String`s,
//! no compiler handles, so they outlive the inference contexts the driver reads them from
//! and are exercised by plain unit tests.

use crate::diagnosis::node::PartialKind;

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
        /// What the field's `help`s can name: whether adding it is confirmed to fix the cause, its
        /// type, and a similar field. It is not part of the leaf's identity (see [`FieldFix`]).
        fix: FieldFix,
    },
    /// A `HasField` bound whose field is present but carries the wrong type: the trait bound holds
    /// and only the `<owner as HasField<name>>::Value == expected` projection fails. The emitter
    /// words this as a `[CGP-E004]` main message of its own.
    FieldTypeMismatch {
        /// The field name, decoded from its `Symbol!`, e.g. `height`.
        name: String,
        /// The struct that carries the field, e.g. `Rectangle`.
        owner: String,
        /// The type the wiring requires the field to have, taken from the failing projection's
        /// right-hand side and rendered *un-normalized*, e.g. `f64` or
        /// `Pool<<App as HasDbType>::Db>`. Keeping the projection is deliberate: it names where the
        /// requirement comes from, so a reader is pointed at the wiring rather than left wondering
        /// why a type they never wrote is demanded of them.
        expected: String,
        /// What `expected` normalizes to, when normalizing changes it and reduces to a concrete
        /// type — `Some("Pool<Postgres>")` for `Pool<<App as HasDbType>::Db>`, and `None` for an
        /// `expected` that is already concrete or does not reduce. The un-normalized form alone
        /// says where the requirement comes from but not what it currently resolves to, and the two
        /// things in conflict are then never put side by side, so both are carried.
        expected_normalized: Option<String>,
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
        /// A key the context wires that is a likely slip of this one, for a `help`.
        similar: NearMiss,
    },
    /// A non-context delegation table that has no entry for a key it is asked to resolve. The owner
    /// is a *provider* that delegates — an [aggregate
    /// provider](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/aggregate-providers.md)
    /// missing a component wiring, or a
    /// [`UseDelegate`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/providers/use_delegate.md)
    /// / `UseInputDelegate` dispatch table missing a branch for the type it dispatches on (a `Code`
    /// fragment or an `Input` value's type). Parallel to [`Leaf::MissingWiring`], but the owner is
    /// a provider table rather than the context: the fix is to add the entry to *that provider's*
    /// table (or feed the stage a type the table already covers), not to wire a component on the
    /// context.
    MissingDispatchEntry {
        /// The key with no entry — a component marker (`BarProviderComponent`) for an aggregate
        /// provider, a redirect path (`@HandlerComponent.Code.Input`) for an aggregate provider
        /// that opens its own components, or a dispatched-on type (`GenericArray<u8, U32>`) for a
        /// `UseDelegate` / `UseInputDelegate` table.
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
    /// A provider that implements the provider trait, but not for the context or the component
    /// arguments the wiring uses it with: a provider written for one context and wired on another,
    /// or a pipeline stage whose input type is not the type the stage before it produces. Distinct
    /// from [`Leaf::NotAProvider`]: the provider does implement the trait, for other arguments, so
    /// the fix is a provider for these arguments, or other arguments for this provider.
    ProviderForOtherArguments {
        /// The provider, e.g. `GreetPerson`.
        provider: String,
        /// The provider trait it implements for other arguments, e.g. `Greeter`.
        provider_trait: String,
        /// The parameter whose argument no impl of the provider accepts: `None` for the context,
        /// or the provider trait's name for a component parameter, e.g. `Input`.
        param: Option<String>,
        /// The arguments the provider's impls take for that parameter, as the impls write them and
        /// in their order, e.g. `["Person"]`.
        provided: Vec<String>,
        /// The argument the wiring gives that parameter, e.g. `Robot`.
        required: String,
    },
    /// A component the context wires through a hand-written `DelegateComponent` impl, with no
    /// `IsProviderFor` impl beside it. `delegate_components!` writes both, and a consumer call needs
    /// the context's `IsProviderFor` as well as its delegate, so a check of the component passes
    /// while every call fails.
    WiringWithoutProviderFor {
        /// The component the context wires, e.g. `GreeterComponent`.
        component: String,
        /// The context, e.g. `App`.
        owner: String,
        /// The provider the hand-written impl delegates to, for the `help`, when the impl is one
        /// the program wrote. Like [`NearMiss`], it is not part of the leaf's identity.
        delegate: EntryFix,
    },
    /// A consumer-trait method called on a value of a generic item's type parameter that the item
    /// does not bound by the consumer trait: the method exists only where the parameter has the
    /// bound, so the call needs it, and the fix is the bound, not wiring.
    UnboundedParameter {
        /// The type parameter, e.g. `Context`.
        param: String,
        /// The consumer trait whose method the call names, e.g. `CanGreet`.
        trait_name: String,
        /// The generic item that declares the parameter, e.g. `run`.
        item: String,
    },
    /// A namespace redirect whose target path has no delegate entry in the context. A
    /// `RedirectLookup<Ctx, Path>` provider resolves a component by forwarding the lookup to `Path`
    /// inside `Ctx`'s wiring (through the namespace lookup trait every joined namespace supplies),
    /// but nothing — no direct entry, no namespace default, no `#[default_impl]` — terminates that
    /// path with a provider, so the redirect resolves to nothing. Parallel to
    /// [`Leaf::MissingWiring`], but keyed by a redirect *path* rather than a bare component marker:
    /// the fix is to add a wiring entry for the path (on the context, or in the namespace it
    /// joins).
    MissingRedirectWiring {
        /// The redirect path with no terminating entry, e.g.
        /// `Path!(@app.finance.types.QuantityTypeProviderComponent)` (resugared from its `PathCons`
        /// spine when the emitter post-processes the note).
        path: String,
        /// The context the lookup is forwarded inside, e.g. `MockApp` — the table that carries no
        /// delegate entry for the path.
        context: String,
        /// A key the context wires that is a likely slip of this path, for a `help`.
        similar: NearMiss,
    },
    /// An associated type the wiring projects through that carries a different type from the one a
    /// provider requires. Like [`Leaf::FieldTypeMismatch`], the trait bound itself holds and only
    /// the `<owner as Trait>::Assoc == expected` projection fails; unlike it, the projection is
    /// *not* `HasField`'s `Value`. The archetype is a CGP [abstract
    /// type](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/abstract-types.md):
    /// the context binds `HasErrorType::Error` to one concrete type by wiring
    /// `ErrorTypeProviderComponent` to `UseType<T>`, while a provider pins it to another with the
    /// `#[use_type(HasErrorType.{Error = AppError})]` equality form. The emitter words this as a
    /// `[CGP-E005]` main message of its own.
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
        /// What `expected` normalizes to, when normalizing changes it and reduces to a concrete
        /// type — `Some("Tx<Postgres>")` for `Tx<<App as HasDbType>::Db>`, and `None` for an
        /// `expected` that is already concrete or does not reduce. A pin whose right-hand side
        /// projects through another abstract type (the `{HashedPassword = Password}` unification
        /// form, or `{Transaction = Tx<Db>}`) produces exactly such a requirement, and naming only
        /// the projection would say where it comes from without saying what it resolves to. Mirrors
        /// [`Leaf::FieldTypeMismatch::expected_normalized`].
        expected_normalized: Option<String>,
        /// The type the owner actually supplies, read by normalizing the projection, e.g. `String`.
        actual: String,
        /// The component marker a context wires to choose this type (`ErrorTypeProviderComponent`),
        /// present only when the trait is a CGP abstract-type component — a consumer trait with one
        /// associated type whose provider carries the `UseType` blanket `#[cgp_type]` generates. It
        /// selects the `abstract type` wording over the plain `associated type` one, and supplies the
        /// wiring `help`; `None` for an ordinary trait's associated type, which has no such fix.
        component: Option<String>,
    },
    /// A field of a record builder that an operation needs unset, which the builder already holds:
    /// a second `build_field`, or a second source of one field in a merge. The builder's partial
    /// type tracks each field's state in a marker, so the operation's `UpdateField` bound finds the
    /// field present where it requires it absent.
    FieldAlreadySet {
        /// The field set twice, e.g. `first_name`.
        name: String,
        /// The struct the builder builds, e.g. `Person`.
        owner: String,
    },
    /// Fields of a record builder that an operation needs set, which the builder lacks: a
    /// `finalize_build` before every field is built, or a `take_field` of a field never built.
    FieldsNotSet {
        /// The unset fields, in declaration order, e.g. `["port"]`.
        names: Vec<String>,
        /// The struct the builder builds, e.g. `Config`.
        owner: String,
    },
    /// A variant an extensible-data operation names that the enum does not declare: a cast whose
    /// other enum lacks one of the variants it moves, or a construction by a tag the enum lacks.
    MissingVariant {
        /// The variant the operation names, e.g. `Bar`.
        name: String,
        /// The enum that lacks it, e.g. `Big`.
        owner: String,
        /// A variant the enum declares whose name is a likely slip of this one, for a `help`.
        similar: NearMiss,
    },
    /// A variant an extractor has already ruled out, extracted again: the remainder of an
    /// extraction that did not yield the variant, asked for it a second time.
    VariantAlreadyExtracted {
        /// The variant extracted again, e.g. `Circle`.
        name: String,
        /// The enum the extractor matches, e.g. `Shape`.
        owner: String,
    },
    /// Variants an extractor still holds when it is finalized. A match extracts variants one at a
    /// time, and the remainder finalizes only once every variant is ruled out, so these variants
    /// are the ones the match never handles.
    VariantsNotExtracted {
        /// The variants not yet extracted, in declaration order, e.g. `["Rectangle"]`.
        names: Vec<String>,
        /// The enum the extractor matches, e.g. `Shape`.
        owner: String,
    },
    /// A variant whose payload is not the type an extensible-data operation requires: a cast
    /// between two enums whose variants of one name carry different payloads. The variant
    /// counterpart of [`Leaf::FieldTypeMismatch`].
    VariantTypeMismatch {
        /// The variant, e.g. `Foo`.
        name: String,
        /// The enum whose variant carries the payload, e.g. `FooBarBaz`.
        owner: String,
        /// The payload type the operation requires, e.g. `u64`.
        expected: String,
        /// The payload type the variant declares, e.g. `u32`.
        actual: String,
    },
    /// A type that does not implement a trait an extensible-data operation needs and a data derive
    /// supplies: `HasFields` (from `#[derive(HasFields)]`), `HasExtractor` (from
    /// `#[derive(ExtractField)]`), `HasBuilder` (from `#[derive(BuildField)]`), and the rest.
    DataTraitUnimplemented {
        /// The type, e.g. `Shape`.
        owner: String,
        /// The trait it does not implement, e.g. `HasFields`.
        trait_name: String,
        /// The derive that implements the trait, named for the `help` when `owner` is a struct or
        /// enum the crate defines (so a derive can be added to it), e.g. `HasFields`. `None` when the
        /// derive cannot be added there. Like [`NearMiss`], it is not part of the leaf's identity.
        derive: DeriveFix,
    },
    /// A field an operation on an optional builder needs held as optional (`set`, `set_optional`),
    /// which the builder holds plainly: the builder is a record's core builder, whose fields are
    /// set or unset, not an optional one.
    FieldNotOptional {
        /// The field, e.g. `foo`.
        name: String,
        /// The struct the builder builds, e.g. `Context`.
        owner: String,
        /// The conversion to an optional builder (`to_optional`), when the builder takes it, for
        /// the `help`. Like [`NearMiss`], it is not part of the leaf's identity.
        convert: MethodFix,
    },
    /// A partial value (a record builder or an enum extractor) given to an operation that needs a
    /// trait the struct or enum itself implements, which no partial value does: `downcast` called on
    /// the remainder of an earlier downcast, where an extractor takes `downcast_fields`.
    PartialLacksDataTrait {
        /// What the partial value is.
        kind: PartialKind,
        /// The struct or enum it builds or matches, e.g. `FooBarBaz`.
        target: String,
        /// The trait it does not implement, e.g. `HasExtractor`.
        trait_name: String,
        /// The method a partial value takes in place of the one called, for the `help`. Like
        /// [`NearMiss`], it is not part of the leaf's identity.
        replacement: MethodFix,
    },
    /// Any other terminal unmet bound — an ordinary trait bound such as `f64: Eq`, or a trait
    /// bound the walk cannot descend further. The emitter keeps rustc's own header for these and
    /// only replaces the sub-notes with the dependency tree.
    Bound {
        /// The bound restated as `self: Trait`, e.g. `f64: std::cmp::Eq`, for the note lead and
        /// for de-duplicating a leaf reached by several paths.
        summary: String,
    },
}

/// What a missing field's `help`s can name. It rides on a [`Leaf::Field`] but is not part of the
/// leaf's identity: every equality and hash treats two values alike, so one missing field reached
/// through requirements that state its type differently stays one cause, and
/// [`Leaf::reconcile_fix`] keeps only what every route agrees on.
#[derive(Debug, Clone, Default)]
pub struct FieldFix {
    /// Whether adding the field is confirmed to fix the cause: its struct implements `HasField` for
    /// another field, so it derives it. A struct with no `HasField` impl may lack the derive too.
    pub confirmed: bool,
    /// The type the requirement gives the field (the `f64` of
    /// `HasField<Symbol!("height"), Value = f64>`), or `None` when no requirement states a concrete
    /// one.
    pub ty: Option<String>,
    /// A field the struct carries whose name is a likely slip of the missing one
    /// ([`similar_name`](super::similar_name)), such as `first_name` for `frist_name`.
    pub similar: Option<String>,
}

/// A key the context wires that is a likely slip of a missing one
/// ([`similar_key`](super::similar_key)), such as `@App.GreeterComponent` for
/// `@app.GreeterComponent`. Like [`FieldFix`], it is carried for a `help` and is not part of the
/// leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct NearMiss(pub Option<String>);

/// The derive a `help` names for a [`Leaf::DataTraitUnimplemented`], when one can be added to the
/// type. Like [`NearMiss`], it is carried for the `help` and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct DeriveFix(pub Option<String>);

/// The provider a `help` names for a [`Leaf::WiringWithoutProviderFor`], to write the entry that
/// replaces a hand-written `DelegateComponent` impl. Like [`NearMiss`], it is carried for the `help`
/// and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct EntryFix(pub Option<String>);

/// A method a `help` tells the reader to call: `to_optional` on a core builder before an optional
/// one's `set`, or `downcast_fields` on an extractor in place of `downcast`. Like [`NearMiss`], it
/// is carried for the `help` and is not part of the leaf's identity.
#[derive(Debug, Clone, Default)]
pub struct MethodFix(pub Option<SuggestedCall>);

/// The method a [`MethodFix`] names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedCall {
    /// The method to call, e.g. `downcast_fields`.
    pub method: String,
    /// The method the call replaces, e.g. `downcast`, or `None` for a call made first.
    pub replaces: Option<String>,
    /// The path that imports the method's trait, e.g. `cgp::core::field::impls::CanDowncastFields`,
    /// since the data traits are not in the prelude. `None` when no public path is found.
    pub trait_path: Option<String>,
}

/// Equality and hashing that treat every value alike, for the facts a leaf carries for its `help`s
/// alone.
macro_rules! identity_free {
    ($ty:ty) => {
        impl PartialEq for $ty {
            fn eq(&self, _: &Self) -> bool {
                true
            }
        }

        impl Eq for $ty {}

        impl std::hash::Hash for $ty {
            fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
        }
    };
}

identity_free!(FieldFix);
identity_free!(NearMiss);
identity_free!(DeriveFix);
identity_free!(EntryFix);
identity_free!(MethodFix);

impl Leaf {
    /// Whether this cause is the state of a field or variant that an extensible-data operation does
    /// not allow (set twice, not set, extracted twice, not extracted, not optional), a variant the
    /// operation names that the enum lacks, or a partial value given where its struct or enum
    /// belongs: a cause read off a partial value's presence markers or a variant list.
    pub fn is_data_state(&self) -> bool {
        matches!(
            self,
            Leaf::FieldAlreadySet { .. }
                | Leaf::FieldsNotSet { .. }
                | Leaf::FieldNotOptional { .. }
                | Leaf::MissingVariant { .. }
                | Leaf::VariantAlreadyExtracted { .. }
                | Leaf::VariantsNotExtracted { .. }
                | Leaf::PartialLacksDataTrait { .. }
        )
    }

    /// Fold the `help` facts of `other`, the same cause reached another way, into this leaf: adding
    /// a field stays confirmed only when both routes confirm it, and a type or a near miss is kept
    /// only when both routes name the same one, since a help naming one route's would not fix the
    /// other.
    pub fn reconcile_fix(&mut self, other: &Leaf) {
        fn agree(mine: &mut Option<String>, theirs: &Option<String>) {
            if mine != theirs {
                *mine = None;
            }
        }
        match (self, other) {
            (Leaf::Field { fix, .. }, Leaf::Field { fix: theirs, .. }) => {
                fix.confirmed &= theirs.confirmed;
                agree(&mut fix.ty, &theirs.ty);
                agree(&mut fix.similar, &theirs.similar);
            }
            (
                Leaf::MissingWiring { similar, .. },
                Leaf::MissingWiring {
                    similar: theirs, ..
                },
            )
            | (
                Leaf::MissingRedirectWiring { similar, .. },
                Leaf::MissingRedirectWiring {
                    similar: theirs, ..
                },
            )
            | (
                Leaf::MissingVariant { similar, .. },
                Leaf::MissingVariant {
                    similar: theirs, ..
                },
            ) => agree(&mut similar.0, &theirs.0),
            (
                Leaf::DataTraitUnimplemented { derive, .. },
                Leaf::DataTraitUnimplemented { derive: theirs, .. },
            ) => agree(&mut derive.0, &theirs.0),
            (
                Leaf::WiringWithoutProviderFor { delegate, .. },
                Leaf::WiringWithoutProviderFor {
                    delegate: theirs, ..
                },
            ) => agree(&mut delegate.0, &theirs.0),
            (
                Leaf::FieldNotOptional { convert: fix, .. },
                Leaf::FieldNotOptional {
                    convert: theirs, ..
                },
            )
            | (
                Leaf::PartialLacksDataTrait {
                    replacement: fix, ..
                },
                Leaf::PartialLacksDataTrait {
                    replacement: theirs,
                    ..
                },
            ) if fix.0 != theirs.0 => fix.0 = None,
            _ => {}
        }
    }
}

// A leaf carries no separate identity key: two causes are the same cause exactly when their leaves
// are equal in every field but the facts carried for a `help`, which is what the derived
// `PartialEq` above gives. An earlier key named only the *thing to fix* (a field or associated-type
// name), which merged two leaves naming the same thing on different owners into one cause whose
// heading could then name only the first of them. Whole-leaf equality keeps the useful merge (one
// leaf genuinely reached by several paths, which is how a diamond survives to the renderer) and
// separates the rest.
