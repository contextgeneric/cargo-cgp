//! A CGP crate whose own code names a lifetime `'__a__`, the spelling a CGP macro reserves.
//!
//! `Labels` declares `'__a__` and leaves its field's lifetime elided, so rustc suggests the
//! `'__a__` lifetime, which is right here, since the program wrote it. The tool drops a suggestion
//! that inserts a reserved name only when the edited file never writes that name, so this one stays
//! and the error matches plain `cargo check`.
//!
//! CGP error class: none; plain Rust the tool passes through.

use cgp::prelude::*;

pub struct Labels<'__a__> {
    pub first: &str,
    pub owner: &'__a__ str,
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
