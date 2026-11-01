//! The `cgp-component` crate and the wiring traits and providers it defines.

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
/// [`CGP_COMPONENT_CRATE`] (its defining crate) to identify the genuine trait: the anchor
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
