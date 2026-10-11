//! Reading a tuple-struct field with `Symbol!("0")` where `Index<0>` belongs.
//!
//! `Pair` derives `HasField`, keyed by `Index<N>`. The tool's help says "missing field `0` on
//! `Pair`", reading the derive as present from rustc's separate "`Pair` implements trait
//! `cgp::prelude::HasField<Tag>`" note, whose path the check strips first. The note's labels show
//! the `Index<0>` and `Index<1>` tags, but nothing says that a tuple field takes an `Index` tag.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

// A tuple field is keyed by `Index<N>`, never by a string of its digit, so `Symbol!("0")` names no
// field of a tuple struct. This is the `Index` page's *Common Mistakes* entry on the two tags.

use cgp::prelude::*;

#[derive(HasField)]
pub struct Pair(pub u32, pub String);

pub fn first(pair: &Pair) -> u32 {
    *pair.get_field(PhantomData::<Symbol!("0")>)
}

fn main() {}
