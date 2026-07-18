//! A component carrying a *lifetime* parameter, failing its check on a missing field.
//!
//! `#[cgp_component]` on a trait with a lifetime keeps the lifetime ahead of the
//! context in the provider trait (`ReferenceGetter<'a, Context, T>`) and lifts it
//! into `Life<'a>` in the check entry's params tuple. The resolver must rebuild the
//! consumer obligation with the lifetime restored to its region slot — not spread
//! `Life<'a>` in as a type — and label the provider node without assuming the
//! context is the trait's first argument.

use cgp::prelude::*;

#[cgp_component(ReferenceGetter)]
pub trait HasReference<'a, T: 'a + ?Sized> {
    fn get_reference(&self) -> &'a T;
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &String;
}

#[cgp_impl(new GetReference)]
impl<'a> ReferenceGetter<'a, str>
where
    Self: HasName,
{
    fn get_reference(&self) -> &'a str {
        let _ = self.name();
        ""
    }
}

#[derive(HasField)]
pub struct App<'a> {
    // missing `name` field to trigger the error
    pub value: &'a str,
}

delegate_components! {
    <'a> App<'a> {
        ReferenceGetterComponent: GetReference,
    }
}

check_components! {
    <'a> App<'a> {
        ReferenceGetterComponent: (Life<'a>, str),
    }
}

fn main() {}
