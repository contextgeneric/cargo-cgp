//! The CGP error codes stamped on rewritten main messages.
//!
//! A CGP error code names one recognized class of CGP mistake. It is attached only when two
//! things are both true: `cargo-cgp` *rewrote* the diagnostic's main message, and that main
//! message was *identified* as a CGP error class (a check-trait failure through
//! `CanUseComponent`, say). The code rides inside the message text as a `[CGP-E001]` prefix —
//! `error[E0277]: [CGP-E001] the consumer trait …` — so the diagnostic's own Rust code
//! (`E0277`, `E0599`) is always kept: the error is still fundamentally rustc's, only restated,
//! and `rustc --explain` stays meaningful.
//!
//! Sub-messages carry no code, even when they are rewritten: renamed obligation notes,
//! resugared `Symbol!`s, and root-cause notes are supporting detail of the one classified
//! error, not classifications of their own. An unrecognized main message is never coded, no
//! matter how much of the rest of the diagnostic was cleaned up.
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
