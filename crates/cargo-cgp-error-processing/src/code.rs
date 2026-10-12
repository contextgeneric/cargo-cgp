//! The CGP error codes stamped on rewritten main messages and dependency-tree entries.
//!
//! A CGP error code names one recognized class of CGP mistake. It rides inside the message text as
//! a `[CGP-Exxx]` prefix (`error[E0277]: [CGP-E001] …`), so the diagnostic's own Rust code
//! (`E0277`, `E0599`) is always kept: the error is still the compiler's, only restated, and
//! `rustc --explain` stays meaningful.
//!
//! The hundreds digit says where a code appears. A **`CGP-E0xx`** code heads a main message, and
//! is attached only when `cargo-cgp` both rewrote the main message and identified it as a CGP error
//! class. The headline codes come in family blocks of ten:
//!
//! - `CGP-E001` to `CGP-E009`: a trait the context cannot use, and the two type mismatches;
//! - `CGP-E010` to `CGP-E019`: conflicting wiring, and structural mistakes;
//! - `CGP-E020` to `CGP-E029`: wiring that never resolves, and the orphan rule;
//! - `CGP-E030` to `CGP-E039`: mistakes in what a macro was given;
//! - `CGP-E040` to `CGP-E049`: wiring keys and values that do not resolve;
//! - `CGP-E050` to `CGP-E059`: extensible data.
//!
//! A **`CGP-E1xx`** code labels an entry of the `root cause:` note's dependency tree, one code per
//! rendering template: `CGP-E101` to `CGP-E109` for the steps of the chain, and `CGP-E110` onward
//! for the root causes it ends at. The `root cause:` line reuses its cause's code, so the line and
//! the tree's last entry carry the same one.
//!
//! The scheme is the letters `CGP-E` followed by three digits, deliberately unlike rustc's `E0277`,
//! so the two namespaces never blur. The catalog of what each code means and how to fix it lives in
//! `cgp-knowledge-base/cargo-cgp/error-code.md`; register a new code there in the same change that
//! adds its constant here.

// A trait the context cannot use, and the two type mismatches (`CGP-E001` to `CGP-E009`).

/// `CGP-E001`: the consumer trait behind a component is not implemented for the context. Stamped
/// on a failed `CanUseComponent` check bound, and on a consumer-method call that fails because the
/// context cannot use a component it wires.
pub const CONSUMER_TRAIT_UNIMPLEMENTED: &str = "CGP-E001";

/// `CGP-E002`: the provider trait behind a component is not implemented for a provider. Stamped on
/// a failed `IsProviderFor` bound (a `#[check_providers(...)]` assertion, or a provider the wiring
/// routes through).
pub const PROVIDER_TRAIT_UNIMPLEMENTED: &str = "CGP-E002";

/// `CGP-E003`: a trait that is not a CGP component, implemented on or required of the context, is
/// not implemented because a CGP component it depends on fails. Unlike
/// [`CONSUMER_TRAIT_UNIMPLEMENTED`], the failing trait is a plain wrapper the programmer wrote (a
/// trait adding a `Send` bound over a CGP consumer supertrait) or a `#[cgp_fn]` blanket trait, so
/// the message says "the trait", not "the consumer trait".
pub const WRAPPER_TRAIT_UNIMPLEMENTED: &str = "CGP-E003";

/// `CGP-E004`: a context field the wiring reads has the wrong type. Stamped on a `type mismatch
/// resolving <Ctx as HasField<Symbol!("f")>>::Value == T` (`E0271`) that the typed resolver traced
/// through CGP wiring to a `HasField` projection: the field is present and derived, but its type
/// does not match the type a provider needs.
pub const FIELD_TYPE_MISMATCH: &str = "CGP-E004";

/// `CGP-E005`: an associated type the context supplies does not match the one a provider requires.
/// Stamped on a `type mismatch resolving <Ctx as HasErrorType>::Error == AppError` (`E0271`) that
/// the typed resolver traced through CGP wiring to a projection other than `HasField`'s, most often
/// a CGP abstract type a context binds one way while a provider pins it another. The sibling of
/// [`FIELD_TYPE_MISMATCH`].
pub const ABSTRACT_TYPE_MISMATCH: &str = "CGP-E005";

// Conflicting wiring (`CGP-E010` to `CGP-E019`): the duplicate-key coherence conflict (`E0119`) a
// `delegate_components!` or `cgp_namespace!` block produces. Each is stamped on the surviving half
// of the conflict pair; they are separate codes because each has its own fix.

/// `CGP-E010`: the same key is wired more than once directly.
pub const DUPLICATE_WIRING: &str = "CGP-E010";

/// `CGP-E011`: two entries wire overlapping but distinct keys, so one cannot claim what the other
/// already covers: a generic entry over a specific one, a key over a namespace forwarding, or a
/// path that covers another.
pub const OVERLAPPING_WIRING: &str = "CGP-E011";

/// `CGP-E012`: a context joins more than one namespace (or a bare-key `for` loop, which lowers the
/// same way), so two blanket forwardings cover every key and overlap.
pub const MULTIPLE_NAMESPACES: &str = "CGP-E012";

/// `CGP-E013`: a direct wiring collides with an `open` or namespace redirect of the same key, so
/// the direct entry never takes effect.
pub const REDIRECT_COLLISION: &str = "CGP-E013";

/// `CGP-E014`: the same key is redirected more than once.
pub const DUPLICATE_REDIRECT: &str = "CGP-E014";

// Structural mistakes in the wiring and its checks, in the same block (`CGP-E015` onward).

/// `CGP-E015`: one `check_components!` table checks the same entry twice. Stamped on the `E0119`
/// between the two identical check impls the table generates.
pub const DUPLICATE_CHECK: &str = "CGP-E015";

/// `CGP-E016`: two `check_components!` tables in one module generate check traits of one name.
/// Stamped on the `E0428` the second definition produces.
pub const DUPLICATE_CHECK_TRAIT: &str = "CGP-E016";

/// `CGP-E017`: a `for` loop in a wiring table whose key does not use the loop's key variable.
/// Stamped on the `E0207` the unconstrained variable produces.
pub const LOOP_KEY_WITHOUT_VARIABLE: &str = "CGP-E017";

/// `CGP-E018`: a `#[default_impl]` on a provider impl that is generic over a parameter the
/// registration cannot carry. Stamped on the `E0207` the parameter produces in the registration.
pub const GENERIC_DEFAULT_IMPL: &str = "CGP-E018";

/// `CGP-E019`: an entry in a wiring table binds generic parameters its key does not use. Stamped on
/// the `E0207` each parameter produces in the impls the entry lowers to.
pub const ENTRY_PARAMETERS_NOT_IN_KEY: &str = "CGP-E019";

// Wiring that never resolves, and the orphan rule (`CGP-E020` to `CGP-E029`).

/// `CGP-E020`: resolving a component's wiring leads back to itself. Stamped on an `E0275` overflow
/// whose requirement is a `CanUseComponent` bound and whose wiring walk meets an obligation again.
pub const WIRING_CYCLE: &str = "CGP-E020";

/// `CGP-E021`: resolving a component's wiring grows without end, each step requiring the consumer
/// trait for a larger type. Stamped on an `E0275` overflow whose requirement is a `CanUseComponent`
/// bound and whose wiring walk reaches its depth limit without a repeat.
pub const GROWING_LOOKUP: &str = "CGP-E021";

/// `CGP-E022`: a namespace registration the orphan rule rejects. Stamped on an `E0210` (or its
/// sibling `E0117`) whose generated impl is a foreign namespace trait implemented for a foreign key.
pub const ORPHAN_FOREIGN_NAMESPACE: &str = "CGP-E022";

// Mistakes in what a macro was given (`CGP-E030` to `CGP-E039`).

/// `CGP-E030`: a CGP trait's method is called in a `#[cgp_fn]` or `#[cgp_impl]` body, but the
/// trait is not declared as a dependency with `#[uses(…)]`. The Rust code stays `E0599`.
pub const UNDECLARED_TRAIT: &str = "CGP-E030";

/// `CGP-E031`: a higher-order provider calls an inner provider (`Inner::method(self)`) that it
/// never imported with `#[use_provider]`. The sibling of [`UNDECLARED_TRAIT`] for an inner
/// provider. The Rust code stays `E0599`.
pub const INNER_PROVIDER_NOT_IMPORTED: &str = "CGP-E031";

/// `CGP-E032`: a `#[cgp_impl]` header names the component's consumer trait where its provider
/// trait belongs. The Rust code stays `E0107`.
pub const CONSUMER_TRAIT_IN_PROVIDER_IMPL: &str = "CGP-E032";

/// `CGP-E033`: `#[cgp_impl]` is applied to a trait that is not a CGP component at all, so there is
/// no provider trait to implement. The Rust code stays `E0107`.
pub const NON_CGP_TRAIT_IN_CGP_IMPL: &str = "CGP-E033";

/// `CGP-E034`: a higher-order provider's inner-provider bound (usually written through
/// `#[use_provider]`) names the component's consumer trait where its provider trait belongs. The
/// Rust code stays `E0107`.
pub const CONSUMER_TRAIT_IN_PROVIDER_BOUND: &str = "CGP-E034";

/// `CGP-E035`: an abstract type is used in a CGP definition (`Self::Error`, or a bare `Error`) that
/// does not import it with `#[use_type(…)]`, so the generated context carries no bound declaring
/// it. The Rust code stays `E0220` (or `E0425` for the bare name).
pub const ABSTRACT_TYPE_NOT_IMPORTED: &str = "CGP-E035";

/// `CGP-E036`: a `#[cgp_impl]` provider does not declare a bound its provider trait requires of
/// the context (from the component's `#[extend(…)]` or `#[use_type(…)]`). The Rust code stays
/// `E0277`.
pub const UNDECLARED_REQUIREMENT: &str = "CGP-E036";

/// `CGP-E037`: `#[uses(…)]` names a component's provider trait where a dependency names its consumer
/// trait. The Rust code stays `E0107`.
pub const PROVIDER_TRAIT_IN_USES: &str = "CGP-E037";

/// `CGP-E038`: `#[cgp_impl(Provider)]` names a provider struct that is never declared, where
/// `#[cgp_impl(new Provider)]` declares it. The Rust code stays `E0425`.
pub const CGP_IMPL_PROVIDER_NOT_DECLARED: &str = "CGP-E038";

// Wiring keys and values that do not resolve (`CGP-E040` to `CGP-E049`).

/// `CGP-E040`: a wiring key names a consumer trait where its component belongs. The Rust code stays
/// `E0782`.
pub const CONSUMER_TRAIT_AS_KEY: &str = "CGP-E040";

/// `CGP-E041`: a wiring key names a component that exists but is not in scope. The Rust code stays
/// `E0425`.
pub const COMPONENT_NOT_IMPORTED: &str = "CGP-E041";

/// `CGP-E042`: a wiring key names no component, and the name points at a consumer trait whose
/// component is named differently. The Rust code stays `E0425`.
pub const KEY_NAMES_NO_COMPONENT: &str = "CGP-E042";

/// `CGP-E043`: a wiring value names a provider that is never declared. The Rust code stays
/// `E0425`.
pub const PROVIDER_NOT_DECLARED: &str = "CGP-E043";

// The steps of a dependency chain (`CGP-E101` to `CGP-E109`).

/// `CGP-E101`: a step through a context's consumer trait impl,
/// `consumer trait impl \`Trait\` for context \`Ctx\``.
pub const DEP_CONSUMER_TRAIT_IMPL: &str = "CGP-E101";

/// `CGP-E102`: a step through a provider trait impl,
/// `provider trait impl \`Trait\` with context \`Ctx\` for provider \`Provider\``.
pub const DEP_PROVIDER_TRAIT_IMPL: &str = "CGP-E102";

/// `CGP-E103`: a step through a namespace or `open` redirect,
/// `redirect lookup to \`@…\` in \`Table\``.
pub const DEP_REDIRECT_LOOKUP: &str = "CGP-E103";

/// `CGP-E104`: a step through any other trait, `trait impl \`Trait\` for \`Type\`` (a user's
/// blanket trait, a getter, or an ordinary bound restated as an impl).
pub const DEP_TRAIT_IMPL: &str = "CGP-E104";

// The root causes a dependency chain ends at (`CGP-E110` onward).

/// `CGP-E110`: a field the wiring reads is absent, `missing field \`f\` on \`T\``.
pub const DEP_MISSING_FIELD: &str = "CGP-E110";

/// `CGP-E111`: a field the struct carries but has not derived `HasField` for.
pub const DEP_UNIMPLEMENTED_ACCESSOR: &str = "CGP-E111";

/// `CGP-E112`: a field present with the wrong type, the tree's face of [`FIELD_TYPE_MISMATCH`].
pub const DEP_FIELD_TYPE_MISMATCH: &str = "CGP-E112";

/// `CGP-E113`: the context wires no provider for a component, or terminates no redirect path.
pub const DEP_MISSING_DELEGATE_ENTRY: &str = "CGP-E113";

/// `CGP-E114`: a provider's own delegation table (an aggregate provider, or a `UseDelegate` or
/// `UseInputDelegate` dispatch table) has no entry for a key it is asked to resolve. Distinct from
/// [`DEP_MISSING_DELEGATE_ENTRY`], whose table is the context.
pub const DEP_MISSING_DISPATCH_ENTRY: &str = "CGP-E114";

/// `CGP-E115`: a type wired where a provider was expected does not implement the provider trait at
/// all, so the fix is to wire an actual provider, not to add an entry.
pub const DEP_NOT_A_PROVIDER: &str = "CGP-E115";

/// `CGP-E116`: an associated type the owner supplies differs from the one required, the tree's face
/// of [`ABSTRACT_TYPE_MISMATCH`].
pub const DEP_ASSOC_TYPE_MISMATCH: &str = "CGP-E116";

/// `CGP-E117`: an ordinary trait bound that does not hold, worded as the compiler words it
/// (`the trait bound \`f64: Eq\` is not satisfied`).
pub const DEP_ORDINARY_BOUND: &str = "CGP-E117";
