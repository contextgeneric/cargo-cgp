#![feature(prelude_import)]
//! A tuple-struct context reached through an implicit argument that reads a named field.
//!
//! The resolver declines and rustc's `E0599` stays, resugared only; the named-field form of the same
//! mistake becomes `[CGP-E003]` over `[CGP-E110]`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Greet {
    fn greet(&self) -> String;
}
impl<__Context__> Greet for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn greet(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}", name))
        })
    }
}
pub struct Person(pub String);
impl HasField<Index<0>> for Person {
    type Value = String;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
impl HasFieldMut<Index<0>> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<0>>,
    ) -> &mut Self::Value {
        &mut self.0
    }
}
fn main() {
    let _ = Person(String::new()).greet();
}
