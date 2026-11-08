#![feature(prelude_import)]
//! A `#[cgp_fn]` trait required by a struct's `where` clause, built with a context missing the field
//! the trait reads. The root cause is the missing `name` field on `App`, and `Holder` only passes the
//! requirement on.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
///The trait for the `get_name` operation.
trait GetName {
    ///The `get_name` operation.
    fn get_name(&self) -> String;
}
impl<__Context__> GetName for __Context__
where
    Self: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{
    fn get_name(&self) -> String {
        let name: &str = ::core::convert::AsRef::as_ref(
            HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("name")>),
        );
        name.to_owned()
    }
}
pub struct Holder<T>(
    pub T,
)
where
    T: GetName;
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
