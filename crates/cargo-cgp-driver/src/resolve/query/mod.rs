//! The compiler readers and solver helpers more than one stage uses. Every module here reads the
//! compiler and decides no resolution, and the resolver's other modules and the class detectors
//! import from here, while nothing here imports them.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/overview.md.

mod argument_ty;
mod binding;
mod byte_range;
mod call_fixed_arguments;
mod calls_at;
mod cgp_item;
mod check_impl;
mod context_param;
mod enclosing_trait_impl;
mod holds;
mod impl_match;
mod impls_declaring_param_at;
mod import_path;
mod keys;
mod local_body_exprs;
mod local_trait_impls;
mod lower;
mod misdirected;
mod owner_has_impl_of;
mod projected_type;
mod projection_mismatch;
mod trait_items_named;
mod traits_with_method;
mod unify_call_inputs;
mod unknowns;
mod written_order;
mod written_ty;

pub use argument_ty::*;
pub use binding::*;
pub use byte_range::*;
pub use call_fixed_arguments::*;
pub use calls_at::*;
pub use cgp_item::*;
pub use check_impl::*;
pub use context_param::*;
pub use enclosing_trait_impl::*;
pub use holds::*;
pub use impl_match::*;
pub use impls_declaring_param_at::*;
pub use import_path::*;
pub use keys::*;
pub use local_body_exprs::*;
pub use local_trait_impls::*;
pub use lower::*;
pub use misdirected::*;
pub use owner_has_impl_of::*;
pub use projected_type::*;
pub use projection_mismatch::*;
pub use trait_items_named::*;
pub use traits_with_method::*;
pub use unify_call_inputs::*;
pub use unknowns::*;
pub use written_order::*;
pub use written_ty::*;
