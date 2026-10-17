//! A `&mut self` auto getter returning `Option<&String>`, which the generated body cannot produce.
//!
//! The error is a plain type mismatch, and rustc's suggestion is spliced after the attribute (a
//! known `cgp` defect), where no edit of the source can apply it; the tool drops it. What remains is
//! the mismatch with short paths (`Option<&mut String>`, not
//! `std::option::Option<&mut std::string::String>`).
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.

//@quality: acceptable
//@codes: none
//@json

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasNickname {
    fn nickname(&mut self) -> Option<&String>;
}

fn main() {}
