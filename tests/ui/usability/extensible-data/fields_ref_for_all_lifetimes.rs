//! A provider bounded on `FieldsRef<'b>` for every lifetime, checked for an enum with a lifetime.
//!
//! The check entry fails with a bare `E0477` "the type `Token<'a>` does not fulfill the required
//! lifetime"; nothing names the for-every-lifetime bound that demands `'static`.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use core::fmt::Debug;

use cgp::prelude::*;

#[cgp_component(ValueShower)]
pub trait CanShowValue<Value> {
    fn show_value(&self, value: &Value) -> String;
}

#[cgp_impl(new ShowFields)]
impl<Value> ValueShower<Value>
where
    Value: ToFieldsRef,
    for<'b> Value::FieldsRef<'b>: Debug,
{
    fn show_value(&self, value: &Value) -> String {
        format!("{:?}", value.to_fields_ref())
    }
}

#[derive(Debug, CgpData)]
pub enum Token<'a> {
    Word(&'a str),
    Number(u64),
}

pub struct App;

delegate_components! {
    App {
        ValueShowerComponent: ShowFields,
    }
}

check_components! {
    <'a> App {
        ValueShowerComponent: Token<'a>,
    }
}

fn main() {}
