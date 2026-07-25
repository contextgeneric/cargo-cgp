//! Resugaring CGP's type-level constructs in expanded Rust source.
//!
//! This crate is the syntax-tree half of `cargo cgp expand`. The driver hands it the source
//! the compiler's pretty-printer produced for a fully expanded crate, and it rewrites every
//! CGP type-level spine back to the surface macro the programmer wrote —
//! `Symbol<6, Chars<'h', …>>` to `Symbol!("height")`, a `Cons` spine to `Product![…]`, a
//! `PathCons` chain to `Path!(@…)` — then prints the result.
//!
//! It links no compiler internals, so it builds and its tests run on any toolchain. That is
//! also why it matches on a parsed [`syn::Type`] rather than on the printed text: the tool's
//! diagnostic resugarers work on strings and are formatting-sensitive, which a printer that
//! line-breaks a long generic list defeats. The full account of what each construct folds
//! back to, and of the three implementations that must agree on it, is
//! `cgp-knowledge-base/cargo-cgp/implementation/resugaring.md`.
//!
//! It also narrows an expansion to one module or item when asked ([`select`]), since an expanded
//! crate is large and a reader usually wants one part of it.
//!
//! The entry point is [`resugar_expanded_source`].

pub mod options;
pub mod resugar;
pub mod select;
pub mod source;

pub use options::ExpandOptions;
pub use resugar::resugar_file;
pub use select::{ItemPath, select_items};
pub use source::resugar_expanded_source;
