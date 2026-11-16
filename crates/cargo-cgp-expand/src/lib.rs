#![doc = include_str!("../README.md")]
// Every public item carries a doc line, which the crate's clippy and doc runs hold.
#![warn(missing_docs)]

pub mod options;
pub mod resugar;
pub mod select;
pub mod source;

pub use options::ExpandOptions;
pub use resugar::resugar_file;
pub use select::{ItemPath, select_items};
pub use source::resugar_expanded_source;
