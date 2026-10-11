//! An enum variant named `Remainder` under `#[derive(CgpData)]`.
//!
//! The extractor slice's `Self::Remainder` collides with the variant, and as for `ExtractField` both
//! notes land on the derive attribute.
//!
//! CGP error class: out-of-scope generated name, its reserved-variant-name sibling
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/out-of-scope-generated-name.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(CgpData)]
pub enum Token {
    Remainder(u64),
    Text(String),
}

fn main() {}
