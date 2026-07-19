//! Rendering a dependency path's predicates as human-readable tree labels.
//!
//! This is where every CGP wiring trait is replaced by the concept it stands for, so the reader
//! never meets a raw `IsProviderFor` or `Symbol`. [`predicate_label`] picks the label template a
//! predicate takes (through the pure constructors in
//! [`cargo_cgp_error_processing::diagnosis`]), and [`render_ty`] renders each type it names,
//! resugaring CGP's type-level spines back to their surface macros. The rendered labels fold into
//! a [`DependencyTree`](cargo_cgp_error_processing::tree::DependencyTree) spine that the
//! [wording](cargo_cgp_error_processing::diagnosis) renders as `cargo tree`-style text.

mod predicate_label;
mod render_ty;

pub(crate) use predicate_label::*;
pub(crate) use render_ty::*;
