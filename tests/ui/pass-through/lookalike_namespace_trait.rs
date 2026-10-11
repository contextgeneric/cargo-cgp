//! A user trait whose only item is `type Delegate`, implemented twice for one type.
//!
//! The two impls conflict (`E0119`), a plain coherence error. The tool must print what plain
//! `cargo check` prints. A lone `type Delegate` is the shape of a CGP namespace trait, but the
//! tool reads it as one only with CGP evidence: an impl keyed by a component marker or a path, or a
//! parent namespace, none of which `Lookup` has.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

#![allow(unused_imports)]

use cgp::prelude::*;

pub trait Lookup<Table> {
    type Delegate;
}

pub struct Key;
pub struct Table;

impl Lookup<Table> for Key {
    type Delegate = u8;
}

impl Lookup<Table> for Key {
    type Delegate = u16;
}

fn main() {}
