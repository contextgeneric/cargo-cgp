//! Well-known names shared with the front-end.

/// Environment variable through which `cargo-cgp` hands us the active toolchain's
/// sysroot. It is the counterpart of `cargo_cgp::config::SYSROOT_ENV`; the two crates
/// declare it independently and the shared string is the contract between them.
pub const SYSROOT_ENV: &str = "CARGO_CGP_SYSROOT";

/// The rustc flag that sets the sysroot. We inject it (with the value from
/// [`SYSROOT_ENV`]) only when cargo has not already passed one, because rustc cannot
/// infer a sysroot from the driver's out-of-tree location.
pub const SYSROOT_FLAG: &str = "--sysroot";

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

/// The item name of the field-access trait whose unmet bound is the root cause the typed
/// resolver reports, paired with [`CGP_FIELD_CRATE`] to confirm a leaf obligation is a
/// genuine CGP `HasField` before decoding its `Symbol!` field name.
pub const HAS_FIELD_TRAIT: &str = "HasField";

/// The crate that defines [`HAS_FIELD_TRAIT`] — distinct from [`CGP_COMPONENT_CRATE`], since
/// `HasField` lives in `cgp-field`, not `cgp-component`.
pub const CGP_FIELD_CRATE: &str = "cgp_field";

/// The crate that defines the type-level string spine (`Symbol`, `Chars`, `Nil`) the resolver
/// walks to decode a field name. Anchoring the decode to this crate keeps a same-named type
/// from another crate from being mistaken for CGP's.
pub const CGP_BASE_TYPES_CRATE: &str = "cgp_base_types";

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
