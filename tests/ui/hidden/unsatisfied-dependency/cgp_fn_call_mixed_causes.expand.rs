#![feature(prelude_import)]
//! A direct call to a `#[cgp_fn]` trait method on a type without wiring, whose blanket impl fails
//! for two kinds of cause: `Describe` requires `Self: Clone`, which `Person` is not, and through
//! `GetName` a `name` field, which `Person` lacks.
//!
//! The call-site anchor resolves the call to the blanket trait, and since the causes are mixed, the
//! headline is the class sentence with a count. `Person` carries no wiring, so the sentence names it
//! plainly, `the trait `Describe` is not implemented for `Person`: 2 root causes`, not as a context.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait GetName {
    fn get_name(&self) -> String;
}
impl<__Context__> GetName for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn get_name(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
pub trait Describe {
    fn describe(&self) -> String;
}
impl<__Context__> Describe for __Context__
where
    Self: Clone,
    Self: GetName,
{
    fn describe(&self) -> String {
        self.get_name()
    }
}
pub struct Person {
    pub age: u8,
}
impl HasField<Symbol!("age")> for Person {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &Self::Value {
        &self.age
    }
}
impl HasFieldMut<Symbol!("age")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &mut Self::Value {
        &mut self.age
    }
}
fn describe_person(person: &Person) -> String {
    person.describe()
}
fn main() {}
