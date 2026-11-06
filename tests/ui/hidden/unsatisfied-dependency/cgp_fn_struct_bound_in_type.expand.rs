#![feature(prelude_import)]
//! A `#[cgp_fn]` trait required by a struct's bound, where the struct is only named in a type, so no
//! expression carries the context. The root cause is the missing `name` field on `App`.
//!
//! The anchors read a context from an expression or from a struct definition, and neither names
//! `App` here, so rustc's diagnostic stays, with its `HasField` clause reworded. It states the cause
//! truly but leads with the bound.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
trait GetName {
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
pub struct Holder<T: GetName>(pub T);
pub struct App {
    pub label: String,
}
impl HasField<Symbol!("label")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &Self::Value {
        &self.label
    }
}
impl HasFieldMut<Symbol!("label")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &mut Self::Value {
        &mut self.label
    }
}
pub struct Outer {
    pub holder: Holder<App>,
}
fn main() {}
