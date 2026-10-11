//! Reading a tuple-struct field with `Symbol!("0")` where `Index<0>` belongs.
//!
//! The tool's help says "`#[derive(HasField)]` is required to access field `0` on `Pair`", but `Pair`
//! derives `HasField`; its fields are keyed by `Index<N>`.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

// A tuple field is keyed by `Index<N>`, never by a string of its digit, so `Symbol!("0")` names no
// field of a tuple struct. This is the `Index` page's *Common Mistakes* entry on the two tags.

use cgp::prelude::*;

#[derive(HasField)]
pub struct Pair(pub u32, pub String);

pub fn first(pair: &Pair) -> u32 {
    *pair.get_field(PhantomData::<Symbol!("0")>)
}

fn main() {}
