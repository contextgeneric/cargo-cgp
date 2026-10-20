//! Well-known names shared with the front-end.

/// Environment variable through which `cargo-cgp` hands us the active toolchain's
/// sysroot. It is the counterpart of `cargo_cgp::config::SYSROOT_ENV`; the two crates
/// declare it independently and the shared string is the contract between them.
pub const SYSROOT_ENV: &str = "CARGO_CGP_SYSROOT";

/// The rustc flag that sets the sysroot. We inject it (with the value from
/// [`SYSROOT_ENV`]) only when cargo has not already passed one, because rustc cannot
/// infer a sysroot from the driver's out-of-tree location.
pub const SYSROOT_FLAG: &str = "--sysroot";

/// Where a crash in the driver is reported: the ICE report rustc prints for a compiler panic names
/// it, and so does the note on a diagnostic the emitter shows untransformed after a panic of its
/// own.
pub const BUG_REPORT_URL: &str = "https://github.com/contextgeneric/cargo-cgp/issues";

/// The flag through which `cargo cgp expand` puts the driver in expand mode, carrying the path
/// the expansion is written to (`--cgp-expand=<path>`).
///
/// It is the counterpart of `cargo_cgp::config::EXPAND_FLAG`; the two crates declare it
/// independently and the shared string is the contract between them. It is not a rustc flag, so
/// [`crate::args::rustc_args`] strips it before the compiler sees the argument vector.
pub const EXPAND_FLAG: &str = "--cgp-expand";

/// The flag through which `cargo cgp expand --item <path>` narrows the expansion to one module or
/// item (`--cgp-expand-item=<path>`).
///
/// The counterpart of `cargo_cgp::config::EXPAND_ITEM_FLAG`, and stripped from the argument vector
/// the same way [`EXPAND_FLAG`] is.
pub const EXPAND_ITEM_FLAG: &str = "--cgp-expand-item";

/// The flag that turns on the next-generation trait solver, injected into every
/// workspace-crate compilation.
///
/// This is how cargo-cgp surfaces the CGP dependency errors the default solver hides.
/// When a provider's impl-side dependency is unmet and the failure is reached by a
/// consumer-method call, the old solver's method-resolution heuristic bottoms out at the
/// provider trait (e.g. `Person: Greeter<Person>`) and never reports the real missing
/// leaf bound (e.g. `Person: HasField<Symbol!("name")>`). The next-gen solver descends to
/// that leaf — and renders CGP's own `#[diagnostic::on_unimplemented]` hint — so simply
/// compiling under it un-hides the root cause. We inject it unless the invocation already
/// sets `-Znext-solver`, so an explicit choice wins.
pub const NEXT_SOLVER_FLAG: &str = "-Znext-solver=globally";

/// The name of the crate that *defines* CGP's wiring traits (`IsProviderFor`,
/// `CanUseComponent`, `DelegateComponent`).
///
/// It is `cgp_component`, not `cgp`: the umbrella `cgp` crate only re-exports these traits
/// through its prelude, and a `pub use` mints no new `DefId`, so the `DefId` the trait solver
/// resolves always belongs to `cgp_component`. The diagnostic-renaming transform checks a
/// candidate `IsProviderFor`'s defining crate against this name before trusting it, so a
/// trait merely *spelled* `IsProviderFor` in some other crate cannot drive the rewrite.
pub const CGP_COMPONENT_CRATE: &str = "cgp_component";

/// The item name of the marker supertrait every CGP provider trait carries. Paired with
/// [`CGP_COMPONENT_CRATE`] to identify the real trait rather than a same-named impostor.
pub const IS_PROVIDER_FOR_TRAIT: &str = "IsProviderFor";

/// The item name of the check trait `check_components!` asserts on a context. Paired with
/// [`CGP_COMPONENT_CRATE`] (its defining crate) to identify the genuine trait — the anchor
/// the typed resolver keys on when it re-runs a check obligation. See `resolve`.
pub const CAN_USE_COMPONENT_TRAIT: &str = "CanUseComponent";

/// The item name of the wiring-table trait. The resolver recognizes it, anchored to
/// [`CGP_COMPONENT_CRATE`], only to *drop* it from the rendered dependency chain: it is pure
/// wiring plumbing that carries no information a reader of the tree needs. It is also the trait
/// whose conflicting impls the duplicate-key conflict classifier reads.
pub const DELEGATE_COMPONENT_TRAIT: &str = "DelegateComponent";

/// The name of the `RedirectLookup` provider type (defined by [`CGP_COMPONENT_CRATE`]). The
/// conflict classifier recognizes a `DelegateComponent` entry whose `Delegate` is a
/// `RedirectLookup<Table, Path>` as a *redirect* (from `open` or a namespace), reading its
/// `Path` (the second argument) to tell the user which redirected key to set.
pub const REDIRECT_LOOKUP_TYPE: &str = "RedirectLookup";

/// The dispatch provider a component wires per parameter value when it declares
/// `#[derive_delegate]`, defined by [`CGP_COMPONENT_CRATE`].
pub const USE_DELEGATE_TYPE: &str = "UseDelegate";

/// The getter provider of a multi-method getter, which reads each method's field by the method's
/// name, defined by [`CGP_COMPONENT_CRATE`].
pub const USE_FIELDS_TYPE: &str = "UseFields";

/// The getter provider that reads one named field, defined by [`CGP_FIELD_CRATE`].
pub const USE_FIELD_TYPE: &str = "UseField";

/// The name of the provider type a context wires to bind an abstract type to a concrete one —
/// `UseType<T>`, the blanket provider `#[cgp_type]` generates so no bespoke provider is needed. The
/// leaf classifier looks for an impl of a provider trait *for* this type to confirm that a
/// single-associated-type consumer trait really is a `#[cgp_type]` abstract-type component, which is
/// what earns the mismatch leaf its `abstract type` wording and its `UseType<…>` wiring `help`.
pub const USE_TYPE_TYPE: &str = "UseType";

/// The crate that defines [`USE_TYPE_TYPE`] (and `HasType`/`TypeProvider`, CGP's built-in
/// abstract-type machinery) — distinct from [`CGP_COMPONENT_CRATE`], since `UseType` lives in
/// `cgp-type`.
pub const CGP_TYPE_CRATE: &str = "cgp_type";

/// The item name of the field-access trait whose unmet bound is the root cause the typed
/// resolver reports, paired with [`CGP_FIELD_CRATE`] to confirm a leaf obligation is a
/// genuine CGP `HasField` before decoding its `Symbol!` field name.
pub const HAS_FIELD_TRAIT: &str = "HasField";

/// The crate that defines [`HAS_FIELD_TRAIT`] — distinct from [`CGP_COMPONENT_CRATE`], since
/// `HasField` lives in `cgp-field`, not `cgp-component`.
pub const CGP_FIELD_CRATE: &str = "cgp_field";

/// The umbrella crate a CGP program depends on and names (`use cgp::prelude::*`). Together with
/// [`CGP_CRATES`], its presence among the crates a compilation uses is the evidence that the
/// compilation's diagnostics can carry CGP constructs at all.
pub const CGP_UMBRELLA_CRATE: &str = "cgp";

/// The crates whose traits count as *CGP evidence* when deciding whether a blanket-impl trait
/// from another crate is a `#[cgp_fn]`/`#[blanket_trait]` blanket trait rather than an ordinary
/// blanket trait.
///
/// A blanket impl over a bare context is the only structural mark such a trait carries, and
/// it is far too broad on its own — `ToString`, `Into`, and `Borrow` all have one. A trait the
/// checked crate *defines* is trusted outright (cargo-cgp runs on CGP workspaces), but a foreign
/// one must show that its blanket actually depends on CGP: on a trait from one of these crates
/// (`HasField` above all), on a CGP consumer trait, or on another trait that does. Anchoring
/// to the defining crates rather than to a name keeps a same-named trait elsewhere from qualifying.
pub const CGP_CRATES: &[&str] = &[
    CGP_COMPONENT_CRATE,
    CGP_FIELD_CRATE,
    CGP_TYPE_CRATE,
    CGP_BASE_TYPES_CRATE,
];

/// The crate that defines the type-level string spine (`Symbol`, `Chars`, `Nil`) the resolver
/// walks to decode a field name. Anchoring the decode to this crate keeps a same-named type
/// from another crate from being mistaken for CGP's.
pub const CGP_BASE_TYPES_CRATE: &str = "cgp_base_types";

/// The name of the type-level path spine (defined by [`CGP_BASE_TYPES_CRATE`]) that `Path!`/`open`
/// wiring expands to. An unmet `DelegateComponent` whose key is a `PathCons` is a redirect *path*
/// the context does not terminate, not a bare component marker, so the leaf classifier renders the
/// whole path rather than reading only its ADT item name (`PathCons`).
pub const PATH_CONS_TYPE: &str = "PathCons";

/// The type-level string `Symbol!` expands to, in `cgp-base-types`, whose `Chars` spine ends in
/// `Nil` as a string terminator rather than an empty list.
pub const SYMBOL_TYPE: &str = "Symbol";

/// The character cell of a `Symbol`'s spine, in `cgp-base-types`.
pub const CHARS_TYPE: &str = "Chars";

/// The name of the type-level empty-path/empty-list terminator (defined by
/// [`CGP_BASE_TYPES_CRATE`]) that closes a `PathCons` spine. The use-site resolver reads it to
/// recognize a simple two-segment `open`-dispatch key `PathCons<Component, PathCons<Value, Nil>>`,
/// so it can recover the real dispatch parameter `Value` rather than re-checking the raw path.
/// It is also the terminator of a `Product!` / `Cons` list, which the tree renderer resugars.
pub const NIL_TYPE: &str = "Nil";

/// The generic parameter `cgp`'s macros end an `@`-path key or redirect target with, the open
/// "rest of the path" a lookup's appended parameters slot into. The programmer never writes it,
/// so a rendered path leaves it out.
pub const WILDCARD_PARAM: &str = "__Wildcard__";

/// The reserved generic parameter `#[cgp_impl]` inserts as the leading context of the provider impl
/// it generates. Its presence is the fingerprint of a macro-generated inside-out impl, telling it
/// apart from a legitimate hand-written direct consumer-trait impl (`impl CanGreet for Person`).
pub const CGP_IMPL_CONTEXT_PARAM: &str = "__Context__";

/// The name of the type-level product-list cell (defined by [`CGP_BASE_TYPES_CRATE`]): a `Product!`
/// expands to a right-nested `Cons<Head, Tail>` spine ended by [`NIL_TYPE`]. The dependency-tree
/// renderer resugars such a spine back to `Product![…]`, anchored by `DefId` to this crate so a
/// same-named type is never resugared.
pub const CONS_TYPE: &str = "Cons";

/// The name of the type-level sum-list cell (defined by [`CGP_FIELD_CRATE`]): a `Sum!` expands to a
/// right-nested `Either<Head, Tail>` spine ended by [`VOID_TYPE`]. The dependency-tree renderer
/// resugars such a spine back to `Sum![…]`, anchored by `DefId` to `cgp-field`.
pub const EITHER_TYPE: &str = "Either";

/// The name of the type-level empty-sum terminator (defined by [`CGP_FIELD_CRATE`]) that closes an
/// `Either` spine — the sum counterpart of [`NIL_TYPE`].
pub const VOID_TYPE: &str = "Void";

/// The name of the type-level lifetime lift (defined by [`CGP_FIELD_CRATE`]): `Life<'a>` stands in
/// for the lifetime `'a` wherever a component's parameters must all be types — the `Params` slot of
/// `CanUseComponent`/`IsProviderFor` and a `check_components!` entry. The resolver reads the region
/// back out of it when it rebuilds a real consumer obligation, whose trait wants the lifetime in a
/// region slot, not a type standing in for one.
pub const LIFE_TYPE: &str = "Life";

/// The name of the type-level field cell (defined by [`CGP_FIELD_CRATE`]): `Field<Tag, Value>`
/// pairs a `Symbol!` name tag or an [`INDEX_TYPE`] position tag with a value type. When every
/// element of a resugared `Product!`/`Sum!` list is a `Field`, the dependency-tree renderer
/// resugars the whole list to the CGP `Struct!`/`Enum!` shape it describes, where the shape has an
/// exact spelling. Anchored by `DefId` to `cgp-field`.
pub const FIELD_TYPE: &str = "Field";

/// The name of the type-level position tag (defined by [`CGP_FIELD_CRATE`]): `Index<N>` tags the
/// `N`th field of a tuple struct or tuple variant inside a [`FIELD_TYPE`] cell, and is what lets a
/// field list resugar to the tuple form `Struct!(A, B)`.
pub const INDEX_TYPE: &str = "Index";

/// The crate that defines CGP's extra extensible-data operations: the optional and defaulted
/// builders (`SetOptional`, `CanBuildWithDefault`, `CanFinalizeWithDefault`, `HasOptionalBuilder`).
/// Its traits, like those of [`CGP_FIELD_CRATE`], are extensible-data traits the walk descends.
pub const CGP_FIELD_EXTRA_CRATE: &str = "cgp_field_extra";

/// The trait (defined by [`CGP_FIELD_CRATE`]) every partial record or enum a data derive generates
/// implements, naming the struct or enum it builds or matches as its `Target`. Its impl is what
/// marks `__PartialPerson<…>` as a partial value rather than an ordinary type.
pub const PARTIAL_DATA_TRAIT: &str = "PartialData";

/// The associated type of [`PARTIAL_DATA_TRAIT`] that names a partial value's struct or enum.
pub const PARTIAL_DATA_TARGET: &str = "Target";

/// The presence-marker trait (defined by [`CGP_FIELD_CRATE`]): a partial value's field or variant
/// is stored as `<M as MapType>::Map<T>`, so its marker parameter `M` records the field's state.
pub const MAP_TYPE_TRAIT: &str = "MapType";

/// The presence markers (defined by [`CGP_FIELD_CRATE`]): a field held (`IsPresent`), not yet set
/// (`IsNothing`), ruled out of an extractor (`IsVoid`), or optional (`IsOptional`).
pub const IS_PRESENT_TYPE: &str = "IsPresent";
/// See [`IS_PRESENT_TYPE`].
pub const IS_NOTHING_TYPE: &str = "IsNothing";
/// See [`IS_PRESENT_TYPE`].
pub const IS_VOID_TYPE: &str = "IsVoid";
/// See [`IS_PRESENT_TYPE`].
pub const IS_OPTIONAL_TYPE: &str = "IsOptional";

/// The borrow markers (defined by [`CGP_FIELD_CRATE`]) a reference extractor's partial enum carries
/// first, saying whether it holds shared (`IsRef`) or mutable (`IsMut`) references to its payloads.
pub const IS_REF_TYPE: &str = "IsRef";
/// See [`IS_REF_TYPE`].
pub const IS_MUT_TYPE: &str = "IsMut";

/// The extensible-data traits (defined by [`CGP_FIELD_CRATE`]) whose unmet obligation the data
/// leaf reads: a builder's per-field update and finalize, an extractor's per-variant extraction and
/// finalize, and an enum's construction by variant.
pub const UPDATE_FIELD_TRAIT: &str = "UpdateField";
/// See [`UPDATE_FIELD_TRAIT`].
pub const FINALIZE_BUILD_TRAIT: &str = "FinalizeBuild";
/// See [`UPDATE_FIELD_TRAIT`].
pub const EXTRACT_FIELD_TRAIT: &str = "ExtractField";
/// See [`UPDATE_FIELD_TRAIT`].
pub const FINALIZE_EXTRACT_TRAIT: &str = "FinalizeExtract";
/// See [`UPDATE_FIELD_TRAIT`].
pub const FROM_VARIANT_TRAIT: &str = "FromVariant";

/// The associated type of [`UPDATE_FIELD_TRAIT`] that names a field's state before the update, and
/// of [`FROM_VARIANT_TRAIT`] that names a variant's payload.
pub const MAPPER_ASSOC: &str = "Mapper";
/// See [`MAPPER_ASSOC`].
pub const VALUE_ASSOC: &str = "Value";

/// The field-access traits (defined by [`CGP_FIELD_CRATE`]) beside its extensible-data
/// operations: reading or writing one field of a value by its tag, which is not an operation on a
/// whole record or enum.
pub const FIELD_ACCESS_TRAITS: &[&str] =
    &["HasField", "HasFieldMut", "FieldGetter", "MutFieldGetter"];

/// The conversion (defined by [`CGP_FIELD_EXTRA_CRATE`]) that turns a record's core builder into an
/// optional one, whose fields `set` and `set_optional` update.
pub const TO_OPTIONAL_TRAIT: &str = "ToOptional";

/// Each extensible-data operation on a struct or enum value paired with its counterpart on a
/// partial value, both defined by [`CGP_FIELD_CRATE`]: a downcast of an enum, and a downcast of an
/// extractor (the remainder of an earlier downcast).
pub const PARTIAL_VALUE_OPERATIONS: &[(&str, &str)] = &[("CanDowncast", "CanDowncastFields")];

/// Each extensible-data trait a type implements only through a derive, paired with that derive:
/// the traits the data leaf reports as unimplemented on a struct or enum, naming the derive to add.
/// All are defined by [`CGP_FIELD_CRATE`].
pub const DATA_DERIVES: &[(&str, &str)] = &[
    ("HasFields", "HasFields"),
    ("HasFieldsRef", "HasFields"),
    ("HasExtractor", "ExtractField"),
    ("HasExtractorRef", "ExtractField"),
    ("HasExtractorMut", "ExtractField"),
    ("HasBuilder", "BuildField"),
    ("IntoBuilder", "BuildField"),
    ("FromVariant", "FromVariant"),
];

/// The stable `--verbose` flag, injected into every workspace-crate compilation to stop
/// the diagnostic machinery from *eliding* the parts of a type it deems uninteresting.
///
/// rustc's error reporting compresses types in several ways that each drop information a
/// downstream consumer of the diagnostic needs, and all of them are gated on the single
/// `opts.verbose` flag that `--verbose` sets:
///
/// - When it reports "trait `X` is not implemented … but trait `Y` is" (the *similar
///   impl* hint), it diffs the two traits and replaces every generic argument the two
///   share with `_`. For CGP that erasure lands inside a `Symbol!` type: two field-name
///   symbols that share a character have that shared `char` printed as `_` in *both*
///   symbols, so the field name cannot be read back from the text at all.
/// - When a type's printed form grows long — routine for CGP's deeply nested `Symbol` /
///   `Cons` spines — it truncates the type and writes the full form to a `long-type-*.txt`
///   file instead of the diagnostic.
/// - When more than nine impls could apply, it prints a few and collapses the rest to
///   "and N others".
///
/// `--verbose` turns all three off, so the full type is always present in the output.
/// Unlike `-Zverbose-internals` (which also flips `opts.verbose`), it does *not* enable
/// the compiler's internal debug printing — no disambiguator suffixes, no region ids — so
/// the diagnostics stay in their ordinary shape, only without the elisions. We inject it
/// unless the invocation already passes `--verbose`, so an explicit choice wins.
pub const VERBOSE_FLAG: &str = "--verbose";
