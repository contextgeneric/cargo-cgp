//! Upcasting into an enum that lacks one of the source's variants.
//!
//! `CanUpcast` widens one enum into another that must cover every variant of it. `Small` has a `Bar`
//! variant that `Big` lacks, so the cast cannot hold. The call-site anchor reads the cast on the
//! constructed `Small` and walks its extractor recursion to the variant:
//! ``[CGP-E050] missing variant `Bar` on `Big` ``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E120

use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(CgpData)]
pub enum Small {
    Foo(u64),
    Bar(String),
}

// `Big` is missing `Small`'s `Bar` variant, so it cannot receive every value of `Small`.
#[derive(CgpData)]
pub enum Big {
    Foo(u64),
}

fn main() {
    let _: Big = Small::Foo(1).upcast(PhantomData::<Big>);
}
