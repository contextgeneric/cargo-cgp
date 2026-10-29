//! The names of CGP's macros, and the names their expansions generate.

/// The name of the `check_components!` macro. An `E0428` arrives from name resolution, before any
/// query can read the crate a macro comes from, so its expansion is recognized by this name, the
/// last segment of the path the call was written with.
pub const CHECK_COMPONENTS_MACRO: &str = "check_components";

/// The name of the `delegate_and_check_components!` macro, recognized like
/// [`CHECK_COMPONENTS_MACRO`]; its check entries write their parameters in `#[check_params(…)]`.
pub const DELEGATE_AND_CHECK_COMPONENTS_MACRO: &str = "delegate_and_check_components";

/// The name of the `#[cgp_auto_dispatch]` attribute macro, recognized like
/// [`CHECK_COMPONENTS_MACRO`] in the `E0428`s of two dispatch traits that share a method name.
pub const CGP_AUTO_DISPATCH_MACRO: &str = "cgp_auto_dispatch";

/// The generic parameter `cgp`'s macros end an `@`-path key or redirect target with, the open
/// "rest of the path" a lookup's appended parameters slot into. The programmer never writes it,
/// so a rendered path leaves it out.
pub const WILDCARD_PARAM: &str = "__Wildcard__";

/// The reserved generic parameter `#[cgp_impl]` inserts as the leading context of the provider impl
/// it generates. Its presence is the fingerprint of a macro-generated inside-out impl, telling it
/// apart from a legitimate hand-written direct consumer-trait impl (`impl CanGreet for Person`).
pub const CGP_IMPL_CONTEXT_PARAM: &str = "__Context__";
