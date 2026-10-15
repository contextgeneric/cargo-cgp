//! Merging two source records that both carry the same field.
//!
//! The second `build_from` brings `bar` again, which the builder already holds. The data-call
//! anchor types the builder after the first merge and walks the second to the field:
//! ``[CGP-E050] field `bar` of `FooBarBaz` is already set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E118

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
