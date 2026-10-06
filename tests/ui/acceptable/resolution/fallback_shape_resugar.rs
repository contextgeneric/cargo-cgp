//! Acceptable: a diagnostic the resolver *declines* still gets its field lists resugared to CGP's
//! `Struct!`/`Enum!` shapes by the fallback post-processing chain.
//!
//! Each `require::<…>()` call asks for a shape to implement `Needs`, which nothing does. There is no
//! CGP wiring behind it, so the resolver declines and the diagnostic falls through to the text
//! post-processing, the same path `fallback_spine_resugar` pins for a plain list. Each shape takes a
//! different rule of the shared spelling rules: the tuple form `Struct!(u8, u16)` read from `Index<N>`
//! tags, a named form with a keyword field written `r#type`, and an `Enum!` whose variants take their
//! shortest spellings (`Empty` for a `Nil` payload, `Pair(u8, u16)` and `Rect { w: f64 }` for a
//! payload that is itself a shape). Every rendered shape is the type the call wrote, so it can be
//! copied back into code.

use cgp::prelude::*;

pub trait Needs {}

fn require<T: Needs>() {}

fn main() {
    require::<Struct!(u8, u16)>();
    require::<Struct! { r#type: u8, name: u16 }>();
    require::<Enum! { Empty, Pair(u8, u16), Rect { w: f64 } }>();
}
