//! Building a tuple struct's field twice, with the tag's index computed as `{ 0 + 1 }`.
//!
//! The data-call anchor reads a const argument only when it is a literal, bare or in braces, since
//! evaluating a constant from inside the emitter could report into the diagnostic it is rewriting.
//! So it cannot type this tag, declines, and the compiler's `E0271` on the builder's `Mapper`
//! stays, printed with the full types the tool's `--verbose` gives, where `Index<1>` writes the same
//! tag and reads as ``[CGP-E050] field `1` of `Pair` is already set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Pair(pub u32, pub u64);

fn main() {
    let _ = Pair::builder()
        .build_field(PhantomData::<Index<{ 0 + 1 }>>, 3)
        .build_field(PhantomData::<Index<{ 0 + 1 }>>, 4);
}
