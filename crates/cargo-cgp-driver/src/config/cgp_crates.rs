//! CGP's umbrella crate and the crates whose items count as evidence of CGP.

use crate::config::{CGP_BASE_TYPES_CRATE, CGP_COMPONENT_CRATE, CGP_FIELD_CRATE, CGP_TYPE_CRATE};

/// The umbrella crate a CGP program depends on and names (`use cgp::prelude::*`). Together with
/// [`CGP_CRATES`], its presence among the crates a compilation uses is the evidence that the
/// compilation's diagnostics can carry CGP constructs at all.
pub const CGP_UMBRELLA_CRATE: &str = "cgp";

/// The path prefix of the items [`CGP_UMBRELLA_CRATE`]'s prelude exports, which a `help` names
/// without a `use`.
pub const CGP_PRELUDE_PATH: &str = "cgp::prelude::";

/// The proc-macro crate that defines `#[cgp_component]`, `#[cgp_impl]`, `cgp_namespace!`, and most of
/// CGP's other macros.
pub const CGP_MACRO_CRATE: &str = "cgp_macro";

/// The crates whose traits count as *CGP evidence* when deciding whether a blanket-impl trait
/// from another crate is a `#[cgp_fn]`/`#[blanket_trait]` blanket trait rather than an ordinary
/// blanket trait.
///
/// A blanket impl over a bare context is the only structural mark such a trait carries, and
/// it is far too broad on its own: `ToString`, `Into`, and `Borrow` all have one. A trait the
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
