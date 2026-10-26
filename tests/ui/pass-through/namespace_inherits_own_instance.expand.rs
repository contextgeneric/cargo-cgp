#![feature(prelude_import)]
//! Two namespaces that each inherit another instantiation of themselves: `Ns<T>: Ns<u32>` and
//! `Ns<T>: Ns<Vec<T>>`.
//!
//! A namespace's every instantiation has the same parent rule, so a lookup that reaches the parent
//! asks the same namespace again, at `u32` forever or at a type that grows, and neither namespace
//! can resolve a key its table lacks. `cgp_namespace!` rejects both at expansion, as it rejects
//! `new Ns: Ns`, with the caret on the parent's name. The tool must print the macro's messages as
//! plain `cargo check` prints them.
//!
//! CGP error class: none; a cycle visible in one invocation, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
fn main() {}
