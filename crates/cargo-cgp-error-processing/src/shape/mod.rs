//! The `Struct!`/`Enum!` record and variant forms a resugared field list renders to.
//!
//! A `Product!` or `Sum!` whose elements are all `Field` cells describes a struct or enum shape,
//! and CGP's `Struct!` and `Enum!` macros write such a shape as the body of a declaration. These
//! functions decide whether a list has such a spelling and render it. They work on names and
//! already-rendered values, so the text post-processing and the driver's typed renderer share them
//! and cannot disagree on which spelling a shape gets. A list with no exact Rust-syntax spelling
//! gets `None`, and the caller keeps its plain `Product!`/`Sum!` form. See
//! `cgp-knowledge-base/cargo-cgp/implementation/resugaring.md`.

mod enum_shape;
mod shape_name;
mod struct_shape;

pub use enum_shape::*;
pub use shape_name::*;
pub use struct_shape::*;
