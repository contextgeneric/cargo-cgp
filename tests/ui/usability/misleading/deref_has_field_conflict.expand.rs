#![feature(prelude_import)]
//! A struct deriving `HasField` for a field its `Deref` target also has.
//!
//! The `E0119` overlap with `cgp`'s `Deref` blanket impl keeps its headline but loses rustc's note
//! naming the blanket impl, and gains a `help` to use `#[derive(HasField)]` over a `[CGP-E108]` root
//! cause saying the impl is missing. The impl exists twice; every added line is false.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::ops::Deref;
use cgp::prelude::*;
pub struct Inner {
    pub name: String,
}
impl HasField<Symbol!("name")> for Inner {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for Inner {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
pub struct Outer {
    pub name: String,
    pub inner: Inner,
}
impl HasField<Symbol!("name")> for Outer {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for Outer {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
impl HasField<Symbol!("inner")> for Outer {
    type Value = Inner;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("inner")>,
    ) -> &Self::Value {
        &self.inner
    }
}
impl HasFieldMut<Symbol!("inner")> for Outer {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("inner")>,
    ) -> &mut Self::Value {
        &mut self.inner
    }
}
impl Deref for Outer {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.inner
    }
}
fn main() {}
