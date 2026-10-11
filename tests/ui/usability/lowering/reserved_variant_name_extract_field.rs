//! An enum variant named `Value` under `#[derive(ExtractField)]`.
//!
//! The generated `Self::Value` is ambiguous with the variant. Both notes land on the derive
//! attribute, so the output never names the variant to rename. This is the opaque case.
//!
//! CGP error class: out-of-scope generated name, its reserved-variant-name sibling
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/out-of-scope-generated-name.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Token {
    Value(u64),
    Text(String),
}

fn main() {}
