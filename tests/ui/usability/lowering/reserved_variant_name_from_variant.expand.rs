#![feature(prelude_import)]
//! An enum variant named `Value` under `#[derive(FromVariant)]`.
//!
//! The readable case: the impl is written for the user's enum, so a note points at the `Value`
//! variant itself.
//!
//! CGP error class: out-of-scope generated name, its reserved-variant-name sibling
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/out-of-scope-generated-name.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub enum Token {
    Value(u64),
    Text(String),
}
impl FromVariant<Symbol!("Value")> for Token {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Value")>,
        value: Self::Value,
    ) -> Self {
        Self::Value(value)
    }
}
impl FromVariant<Symbol!("Text")> for Token {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Text")>,
        value: Self::Value,
    ) -> Self {
        Self::Text(value)
    }
}
fn main() {}
