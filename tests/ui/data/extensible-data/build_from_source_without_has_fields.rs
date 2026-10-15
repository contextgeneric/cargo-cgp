//! Merging a record whose source type lacks `#[derive(HasFields)]`.
//!
//! `CanBuildFrom` walks the source's field list, so the source needs `HasFields` as well as a
//! builder. rustc reports only "`CanBuildFrom<FooBar>` is not implemented" for the target's
//! builder; the tool names the source's missing trait and the derive that supplies it:
//! ``[CGP-E050] `FooBar` does not implement `HasFields` ``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E123

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
