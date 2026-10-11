//! Merging a record whose source type lacks `#[derive(HasFields)]`.
//!
//! `CanBuildFrom` walks the source's field list, so the source needs `HasFields` as well as a builder.
//! Under the pinned nightly the output is only "`CanBuildFrom<FooBar>` is not implemented" for the
//! target's builder; nothing names `HasFields`, which the reference documents quote as the error.
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

#[derive(BuildField)]
pub struct FooBar {
    pub foo: u64,
    pub bar: String,
}

#[derive(BuildField)]
pub struct FooBarBaz {
    pub foo: u64,
    pub bar: String,
    pub baz: bool,
}

fn main() {
    let _ = FooBarBaz::builder()
        .build_from(FooBar { foo: 1, bar: "bar".into() })
        .build_field(PhantomData::<Symbol!("baz")>, true)
        .finalize_build();
}
