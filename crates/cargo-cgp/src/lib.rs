#![doc = include_str!("../README.md")]
// Every public item carries a doc line, which the crate's clippy and doc runs hold.
#![warn(missing_docs)]

pub mod args;
pub mod check;
pub mod config;
pub mod exit;
pub mod expand;
pub mod help;
pub mod launch;
pub mod report;
pub mod run;
pub mod setup;
pub mod stdout;
pub mod toolchain;
pub mod update;
