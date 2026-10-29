//! Recognizing a wiring key, a wiring value, or a `#[cgp_impl]` provider that does not resolve.
//!
//! A consumer trait written as a key fails while the compiler lowers the generated impl's header
//! (`E0782`), so it is recognized as it is emitted ([`detect_consumer_trait_as_key`]). A name that
//! resolves to nothing fails during name resolution (`E0425`), before the HIR exists, so those are
//! recognized together once resolution is over ([`plan_unresolved_names`]), from the driver's
//! `after_expansion` callback. [`slot`] finds the entry or header a name sits in, [`component`] and
//! [`declared`] look names up across the crate graph, and [`unprovable`] lists what a wiring entry
//! with an unresolved value leaves unprovable.

mod component;
mod consumer_trait_key;
mod declared;
mod plan;
mod slot;
mod unprovable;

pub use consumer_trait_key::*;
pub use declared::*;
pub use plan::*;
pub use unprovable::*;
