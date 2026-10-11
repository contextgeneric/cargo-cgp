//! Merging two source records that both carry the same field.
//!
//! The second `build_from` brings `bar` again, which the builder already holds. The output is an
//! `E0277` on `FieldsBuilder` over `Struct! { bar: String, baz: bool }`; it never says `bar` is set
//! twice. The reference documents quote an `E0271` on `Mapper == IsNothing` instead.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(CgpData)]
pub struct BarBaz {
    pub bar: String,
    pub baz: bool,
}

#[derive(CgpData)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

fn main() {
    let _: FooBarBaz = FooBarBaz::builder()
        .build_from(FooBar { foo: 1, bar: "bar".into() })
        .build_from(BarBaz { bar: "bar".into(), baz: true })
        .finalize_build();
}
