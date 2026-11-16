//! Classifying a duplicate-key wiring conflict by querying the trait solver: the redundant
//! `IsProviderFor` half is dropped and the `DelegateComponent` half names the colliding keys.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md.

mod build;
mod classify;
mod delegate_impls;
mod namespace;
mod overlap;
mod redirect;
mod target;

pub use build::*;
pub use classify::*;
pub use delegate_impls::*;
pub use namespace::*;
pub use overlap::*;
pub use redirect::*;
pub use target::*;
