//! Building a tuple struct's field twice, with the tag's index written in braces.
//!
//! `Index<{ 0 }>` is the same tag as `Index<0>`: the braces make the index an anonymous constant
//! whose body is the literal. The data-call anchor reads a literal const argument through the
//! braces, so it types the builder's tag and reports the second `build_field` as the field set
//! twice: ``[CGP-E050] field `0` of `Pair` is already set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E118

#![allow(unused_braces)]

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Pair(pub u32, pub u64);

fn main() {
    let _ = Pair::builder()
        .build_field(PhantomData::<Index<{ 0 }>>, 1)
        .build_field(PhantomData::<Index<{ 0 }>>, 2);
}
