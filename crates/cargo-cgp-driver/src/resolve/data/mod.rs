//! Recognizing CGP's extensible-data traits and reading the partial values they operate on.
//!
//! A record builder or an enum extractor is a *partial value*: a type a data derive generates
//! (`__PartialPerson<IsPresent, IsNothing>`) whose marker parameters record each field's or
//! variant's state. `traits` recognizes the data traits the walk descends and the list recursions
//! it drops, and `partial` reads a partial value's markers back into the field or variant each
//! tracks, which the [classifier](crate::resolve::classify) words as the data leaf and the
//! [labels](crate::resolve::label) name in words. See
//! `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution/leaves.md`.

mod partial;
mod traits;

pub use partial::*;
pub use traits::*;
