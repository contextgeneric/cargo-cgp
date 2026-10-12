#![feature(prelude_import)]
//! Reading a tuple-struct field with `Symbol!("0")` where `Index<0>` belongs.
//!
//! `Pair` derives `HasField`, keyed by `Index<N>`. The help keeps rustc's "the trait
//! `HasField<Symbol!("0")>` is not implemented for `Pair`", since a digit tag names no field. The
//! note's labels show the `Index<0>` and `Index<1>` tags, but nothing says that a tuple field takes
//! an `Index` tag.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Pair(pub u32, pub String);
impl HasField<Index<0>> for Pair {
    type Value = u32;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
impl HasFieldMut<Index<0>> for Pair {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<0>>,
    ) -> &mut Self::Value {
        &mut self.0
    }
}
impl HasField<Index<1>> for Pair {
    type Value = String;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<1>>) -> &Self::Value {
        &self.1
    }
}
impl HasFieldMut<Index<1>> for Pair {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<1>>,
    ) -> &mut Self::Value {
        &mut self.1
    }
}
pub fn first(pair: &Pair) -> u32 {
    *pair.get_field(PhantomData::<Symbol!("0")>)
}
fn main() {}
