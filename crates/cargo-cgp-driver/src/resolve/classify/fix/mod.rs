//! The fix facts a leaf's `help` names, read once the leaf is classified.
//!
//! [`required_type`] reads the type a requirement gives a missing field, [`near_miss`] finds a
//! similar field or wiring key the program wrote, [`other_arguments`] names the argument a provider
//! does not implement its provider trait for, [`hand_wiring`] recognizes a component wired by a
//! hand-written `DelegateComponent` impl without its `IsProviderFor`, and the hints for a type
//! wired where a provider belongs: [`adapter`] names the adapter that makes it one,
//! [`provided_components`](mod@provided_components) the other components it provides, and
//! [`core_provider`] what serves the slot a misused CGP core provider cannot.

mod adapter;
mod core_provider;
mod hand_wiring;
mod near_miss;
mod other_arguments;
mod provided_components;
mod required_type;

pub use adapter::*;
pub use core_provider::*;
pub use hand_wiring::*;
pub use near_miss::*;
pub use other_arguments::*;
pub use provided_components::*;
pub use required_type::*;
