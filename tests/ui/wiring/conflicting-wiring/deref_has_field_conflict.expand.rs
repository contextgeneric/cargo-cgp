#![feature(prelude_import)]
//! A struct deriving `HasField` for a field its `Deref` target also has.
//!
//! The derive's `HasField` impl overlaps `cgp`'s `Deref` blanket impl (`E0119`). The tool keeps
//! rustc's diagnostic, with the field tag resugared in the headline and rustc's note naming the
//! conflicting blanket impl. The resolver takes only an unmet obligation, and an overlap is not one,
//! so it adds no missing-derive help or cause.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
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
