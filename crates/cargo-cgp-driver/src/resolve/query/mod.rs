//! The compiler readers and solver helpers more than one stage uses.
//!
//! Every module here reads the compiler and decides no resolution:
//!
//! - **Solver queries** the walk and the classifiers ask: [`holds`](fn@holds),
//!   [`impl_where_obligations`], [`projection_mismatch`](fn@projection_mismatch),
//!   [`projected_type`](fn@projected_type), and the placeholder folders such as
//!   [`unknowns_to_placeholders`].
//! - **The impl match** the walk and the readers of a matched impl share, [`match_impl`] over
//!   [`fresh_impl_ref`], and the **readers of an impl's clauses:**
//!   [`written_order`](fn@written_order), [`written_order_clauses`], and
//!   [`misdirected_impl_clauses`].
//! - **The unification of a call's arguments** with its callee's declared inputs, which the
//!   call-site anchors and the detectors share: [`unify_call_inputs`](fn@unify_call_inputs).
//! - **Renderings of a wiring key**, its path, and its provider: [`describe_key`] and its siblings.
//! - **Syntactic type reads** the call-site anchors and the detectors share: [`lower_hir_ty`] and
//!   [`expr_written_ty`], and a binding's declared type, as written ([`declared_binding_ty`]) or as
//!   its signature gives it ([`param_declared_input`], [`argument_ty`](fn@argument_ty)).
//! - **Lookups** several stages make: [`check_impls_at`] over [`can_use_component_supertrait`] and
//!   [`impl_self_ty_span`], [`context_param_span`],
//!   [`enclosing_trait_impl`](fn@enclosing_trait_impl), [`import_path`](fn@import_path),
//!   [`local_body_exprs`](fn@local_body_exprs), [`local_trait_impls`](fn@local_trait_impls),
//!   [`owner_has_impl_of`](fn@owner_has_impl_of), [`trait_items_named`](fn@trait_items_named), and
//!   [`traits_with_method`](fn@traits_with_method).
//!
//! The resolver's other modules and the class detectors import from here, and nothing here imports
//! them.

mod argument_ty;
mod binding;
mod cgp_item;
mod check_impl;
mod context_param;
mod enclosing_trait_impl;
mod holds;
mod impl_match;
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
pub use cgp_item::*;
pub use check_impl::*;
pub use context_param::*;
pub use enclosing_trait_impl::*;
pub use holds::*;
pub use impl_match::*;
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
