//! The `cgp-type` crate and the abstract-type provider it defines.

/// The name of the provider type a context wires to bind an abstract type to a concrete one:
/// `UseType<T>`, the blanket provider `#[cgp_type]` generates so no bespoke provider is needed. The
/// leaf classifier looks for an impl of a provider trait *for* this type to confirm that a
/// single-associated-type consumer trait really is a `#[cgp_type]` abstract-type component, which is
/// what earns the mismatch leaf its `abstract type` wording and its `UseType<…>` wiring `help`.
pub const USE_TYPE_TYPE: &str = "UseType";

/// The crate that defines [`USE_TYPE_TYPE`] (and `HasType`/`TypeProvider`, CGP's built-in
/// abstract-type machinery), distinct from [`CGP_COMPONENT_CRATE`](super::CGP_COMPONENT_CRATE),
/// since `UseType` lives in `cgp-type`.
pub const CGP_TYPE_CRATE: &str = "cgp_type";
