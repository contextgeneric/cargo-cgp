//! The CGP error codes stamped on rewritten main messages and dependency-tree entries.
//!
//! A CGP error code names one recognized class of CGP mistake. It rides inside the message text
//! as a `[CGP-Exxx]` prefix — `error[E0277]: [CGP-E001] the consumer trait …` — so the
//! diagnostic's own Rust code (`E0277`, `E0599`) is always kept: the error is still
//! fundamentally rustc's, only restated, and `rustc --explain` stays meaningful.
//!
//! The three-digit space is split by *what* the code classifies. The **`CGP-E0xx`** range is for
//! **main messages** — the diagnostic's headline. A main-message code is attached only when
//! `cargo-cgp` both *rewrote* the main message and *identified* it as a CGP error class (a
//! check-trait failure through `CanUseComponent`, say); an unrecognized main message is never
//! coded, no matter how much of the rest of the diagnostic was cleaned up. The **`CGP-E1xx`**
//! range is for **dependency-tree entries** — each node of the `root cause:` note's `cargo
//! tree`, one code per distinct rendering template, so `consumer trait impl \`C\` for context
//! \`Ctx\`` and `provider trait impl \`P\` …` carry different codes. A tree entry that merely
//! *passes through* a non-CGP message (the ordinary-bound restatement `the trait bound \`f64:
//! Eq\` is not satisfied`) carries no code; a tree entry the tool *rewrote* into its own
//! template — including the general `trait impl \`Trait\` for \`Type\`` — does. The **`CGP-E2xx`**
//! range is for **the `root cause:` note lead**: it reuses the code of the terminal leaf it names
//! (so a missing-field root cause reads `[CGP-E106]`), and takes a fresh `CGP-E2xx` code only where
//! the leaf itself is an uncoded pass-through bound and so has no code to reuse.
//!
//! Renamed obligation notes and resugared `Symbol!`s are supporting detail, not classifications
//! of their own, and stay uncoded.
//!
//! The scheme is the letters `CGP-E` followed by three digits, deliberately unlike rustc's
//! `E0277`, so the two namespaces never blur. The catalog of what each code means and how to
//! fix it lives in `docs/error-code.md`; register a new code there in the same change that
//! adds its constant here.

/// `CGP-E001` — the consumer trait behind a component is not implemented for the context.
/// Stamped on a failed `CanUseComponent` check bound, and on a consumer-method call that
/// fails because the context cannot use a component it wires.
pub const CONSUMER_TRAIT_UNIMPLEMENTED: &str = "CGP-E001";

/// `CGP-E002` — the provider trait behind a component is not implemented for a provider.
/// Stamped on a failed `IsProviderFor` bound (a `#[check_providers(...)]` assertion, or a
/// provider the wiring routes through).
pub const PROVIDER_TRAIT_UNIMPLEMENTED: &str = "CGP-E002";

/// `CGP-E003` — a context field the wiring reads has the wrong type. Stamped on a `type mismatch
/// resolving <Ctx as HasField<Symbol!("f")>>::Value == T` (`E0271`) that the typed resolver traced
/// through CGP wiring to a `HasField` projection: the field is present and derived, but its type
/// does not match the type a provider needs. The rewritten message names the field, its expected
/// type, and the actual type found on the struct.
pub const FIELD_TYPE_MISMATCH: &str = "CGP-E003";

// The `CGP-E004`–`CGP-E008` family covers the duplicate-key coherence conflict (`E0119`) a
// `delegate_components!` block produces. All five are stamped on the `DelegateComponent` half of
// the conflict pair (its redundant `IsProviderFor` half is dropped); they are separate codes
// because each rewrites the message into a distinct form with its own fix.

/// `CGP-E004` — the same key is wired more than once directly. The same component marker, or the
/// same `@`-path, mapped twice on one context.
pub const DUPLICATE_WIRING: &str = "CGP-E004";

/// `CGP-E005` — two entries wire *overlapping but distinct* keys, so one cannot claim what the
/// other already covers: a generic over a specific, a bare key or `@`-path over a namespace
/// forwarding, or a path that is a prefix of another.
pub const OVERLAPPING_WIRING: &str = "CGP-E005";

/// `CGP-E006` — a context joins more than one namespace (or a bare-key `for` loop, which desugars
/// the same way), so two blanket forwardings cover every key and overlap. Only one namespace can
/// key each target type.
pub const MULTIPLE_NAMESPACES: &str = "CGP-E006";

/// `CGP-E007` — a direct wiring collides with an `open`/namespace redirect of the same key, so the
/// direct entry never takes effect; the fix is to set the redirected path instead.
pub const REDIRECT_COLLISION: &str = "CGP-E007";

/// `CGP-E008` — the same key is redirected more than once (two `open`s, or two `=>` mappings).
pub const DUPLICATE_REDIRECT: &str = "CGP-E008";

/// `CGP-E009` — a hand-written trait implemented directly on a context is not implemented, because a
/// CGP component it depends on fails. Unlike [`CONSUMER_TRAIT_UNIMPLEMENTED`], the failing trait is
/// *not* a CGP consumer trait — it is a plain wrapper the programmer wrote (the transfer example's
/// `CanHandleApiSend`, which adds a `Send` bound over a CGP consumer supertrait) — so the message
/// says "the trait", not "the consumer trait". The root cause is a CGP failure the resolver traced
/// through the wrapper's supertrait, shown in the `root cause:` note.
pub const WRAPPER_TRAIT_UNIMPLEMENTED: &str = "CGP-E009";

/// `CGP-E010` — resolving a component's wiring never terminates. Stamped on an `E0275` overflow
/// whose requirement is a `CanUseComponent` bound: the lookup recurses without bottoming out,
/// which almost always means the wiring routes the component back to the context itself — a
/// component delegated to `UseContext` whose only implementation *is* that delegation. The fix is
/// to wire a real provider, or to implement the consumer trait directly on the context.
pub const WIRING_OVERFLOW: &str = "CGP-E010";

/// `CGP-E011` — a namespace registration the orphan rule rejects. Stamped on an `E0210` (or its
/// sibling `E0117`) whose generated impl is a foreign namespace trait implemented for a foreign
/// key: a `#[default_impl]` / `#[prefix]` registering a foreign component into a foreign namespace,
/// or a `cgp_namespace!` block re-opening one. The rewritten message names the foreign namespace
/// and key instead of the machinery parameter (`__Components__` / `__Table__`), and the fix — own
/// one end of the wiring — rides in a `help`.
pub const ORPHAN_FOREIGN_NAMESPACE: &str = "CGP-E011";

/// `CGP-E012` — a CGP capability is called in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared as a
/// dependency. The macro lowers the body into a blanket impl over a generated generic context
/// (`impl<__Context__> Describe for __Context__ where __Context__: GetName`), and a capability the
/// body calls on `self` must be a `where` bound on that context — declared with `#[uses(…)]`. When
/// it is omitted, rustc reports a vague `E0599` on `&__Context__` pointing at a transitive
/// `HasField` bound; the rewritten message names the capability instead, and the fix — add it to
/// `#[uses(…)]` — rides in a `help`. The Rust code stays `E0599`.
pub const UNDECLARED_CAPABILITY: &str = "CGP-E012";

// The `CGP-E1xx` family codes the entries of a `root cause:` note's dependency tree, one code per
// distinct rendering template. `CGP-E101`, `CGP-E102`, `CGP-E104`, and `CGP-E105` are the inner chain
// nodes (a wiring hop — `CGP-E103` is retired, see below); `CGP-E106`–`CGP-E111` are the terminal
// root-cause leaves. An entry that passes a non-CGP message through unchanged (the `the trait bound …
// is not satisfied` restatement) carries no code.

/// `CGP-E101` — a dependency-chain hop through a context's *consumer* trait impl:
/// `consumer trait impl \`Trait\` for context \`Ctx\``.
pub const DEP_CONSUMER_TRAIT_IMPL: &str = "CGP-E101";

/// `CGP-E102` — a dependency-chain hop through a *provider* trait impl:
/// `provider trait impl \`Trait\` with context \`Ctx\` for provider \`Provider\``.
pub const DEP_PROVIDER_TRAIT_IMPL: &str = "CGP-E102";

// `CGP-E103` is retired: it named a mid-chain `HasField` accessor hop, but a `HasField` obligation is
// always a terminal root-cause leaf in the walk (coded `CGP-E106`/`CGP-E108`), never an interior hop,
// so the code was never emitted. The number is left unused rather than reassigned, so the others stay
// stable.

/// `CGP-E104` — a dependency-chain hop through a namespace/`open` redirect:
/// `redirect lookup to \`@…\` in \`Ctx\``.
pub const DEP_REDIRECT_LOOKUP: &str = "CGP-E104";

/// `CGP-E105` — a dependency-chain hop through any other trait, rendered in the general
/// `trait impl \`Trait\` for \`Type\`` form (a user capability trait, or an ordinary bound
/// restated as an impl).
pub const DEP_TRAIT_IMPL: &str = "CGP-E105";

/// `CGP-E106` — a root-cause leaf: a context field the wiring reads is genuinely absent
/// (`missing field \`f\` on \`T\``).
pub const DEP_MISSING_FIELD: &str = "CGP-E106";

/// `CGP-E107` — a root-cause leaf: the context wires no provider for a component (or terminates
/// no namespace path), `context \`Ctx\` does not contain any delegate entry for \`key\``.
pub const DEP_MISSING_DELEGATE_ENTRY: &str = "CGP-E107";

/// `CGP-E108` — a root-cause leaf: a field the struct carries but has not derived `HasField` for,
/// `accessor trait \`HasField\` with field \`f\` is not implemented for \`T\``.
pub const DEP_UNIMPLEMENTED_ACCESSOR: &str = "CGP-E108";

/// `CGP-E109` — a root-cause leaf: a field present with the wrong type,
/// `field \`f\` on \`T\` has type \`A\`, but \`B\` is required`.
pub const DEP_FIELD_TYPE_MISMATCH: &str = "CGP-E109";

/// `CGP-E110` — a root-cause leaf: a non-context delegation table — an aggregate provider, or a
/// `UseDelegate`/`UseInputDelegate` dispatch table — has no entry for a key it is asked to resolve,
/// `provider \`T\` does not contain any delegate entry for \`Key\``. Distinct from
/// [`DEP_MISSING_DELEGATE_ENTRY`] (a component the *context* does not wire): here the owner is a
/// provider table, so the fix is to add the entry to that provider (or feed the stage a type/key the
/// table already covers).
pub const DEP_MISSING_DISPATCH_ENTRY: &str = "CGP-E110";

/// `CGP-E111` — a root-cause leaf: a type wired where a *provider* was expected does not implement
/// the provider trait at all, `the provider trait \`T\` is not implemented for \`X\``. The mistake is
/// wiring a non-provider (often a request/value type) into a provider slot — e.g.
/// `UseBasicAuth<QueryBalanceRequest>` where the endpoint handler is missing — so the fix is to use an
/// actual provider, not to add a wiring entry (which is [`DEP_MISSING_DISPATCH_ENTRY`]).
pub const DEP_NOT_A_PROVIDER: &str = "CGP-E111";

// The `CGP-E2xx` range codes the `root cause:` note lead. It reuses the terminal leaf's `CGP-E1xx`
// code where the leaf has one; the constants here cover only the leads that need a code of their
// own because the leaf they name is an uncoded pass-through.

/// `CGP-E201` — the `root cause:` lead for a leaf that is an ordinary (non-CGP) trait bound: the
/// leaf itself passes through uncoded (`the trait bound \`f64: Eq\` is not satisfied`), but the
/// lead still names a classified root cause, so it takes this code.
pub const ROOT_CAUSE_ORDINARY_BOUND: &str = "CGP-E201";
