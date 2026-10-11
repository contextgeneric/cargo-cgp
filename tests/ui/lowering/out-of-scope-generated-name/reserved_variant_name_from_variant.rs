//! An enum variant named `Value` under `#[derive(FromVariant)]`.
//!
//! The readable case: the impl is written for the user's enum, so a note points at the `Value`
//! variant itself.
//!
//! CGP error class: out-of-scope generated name, its reserved-variant-name sibling
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/out-of-scope-generated-name.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(FromVariant)]
pub enum Token {
    Value(u64),
    Text(String),
}

fn main() {}
