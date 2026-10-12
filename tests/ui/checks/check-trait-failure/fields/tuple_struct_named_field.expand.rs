#![feature(prelude_import)]
//! A tuple-struct context reached through an implicit argument that reads a named field.
//!
//! The call-site anchor reads `Person` off the tuple-struct constructor and resolves the call to
//! ``[CGP-E003] missing field `name` on `Person` ``, as it does the named-field form of the same
//! mistake. No help names a fix: a tuple struct cannot gain a named field, so the help that adds
//! the field to a struct with named fields does not apply.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
