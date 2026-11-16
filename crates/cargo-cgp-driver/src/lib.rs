#![doc = include_str!("../README.md")]
#![feature(rustc_private)]
// Every public item carries a doc line, which the crate's clippy and doc runs hold.
#![warn(missing_docs)]

// The compiler's internal libraries come from the sysroot through `rustc_private`, which needs a
// nightly with the `rustc-dev` component. Add an `extern crate rustc_*;` line here when a module
// needs a further compiler crate.
extern crate rustc_ast;
extern crate rustc_ast_pretty;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_infer;
extern crate rustc_interface;
extern crate rustc_lint_defs;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_trait_selection;

pub mod args;
pub mod callbacks;
pub mod component_map;
pub mod config;
pub mod detect;
pub mod emitter;
pub mod expand;
pub mod help;
pub mod resolve;
pub mod run;
pub mod version;
