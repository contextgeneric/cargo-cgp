//! Upcasting from an enum that derives only `ExtractField`.
//!
//! An upcast walks the source's `HasFields` list, so the source needs `#[derive(HasFields)]` too.
//! rustc reports "no method named `upcast` found for enum `Small`"; the tool names the missing
//! trait and its derive: ``[CGP-E050] `Small` does not implement `HasFields` ``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E123

use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(ExtractField)]
pub enum Small {
    Foo(u64),
}

#[derive(CgpData)]
pub enum Big {
    Foo(u64),
    Bar(String),
}

fn main() {
    let _ = Small::Foo(1).upcast(PhantomData::<Big>);
}
