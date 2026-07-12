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
