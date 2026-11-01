#![feature(prelude_import)]
//! A `#[cgp_fn]` trait required by a **struct's** bound, built with a context missing the field the
//! trait reads.
//!
//! `#[cgp_fn]` turns `get_name` into the blanket-impl trait `GetName`, which any type with a `name`
//! field implements. `Holder<T: GetName>` requires it of what it holds, so building `Holder(app)`
//! for an `App` without that field fails with an `E0277` on the argument, `required by a bound in
//! `Holder``. The root cause is the missing `name` field on `App`.
//!
//! rustc's note lands on the bound inside `Holder`'s generics, so `Holder` is only passing the
//! requirement on, and the context is the argument the caret points at.
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
fn main() {
    let app = App { label: String::new() };
    let _holder = Holder(app);
}
