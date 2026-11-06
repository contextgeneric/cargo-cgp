//! A `#[cgp_impl]` provider body that calls a trait through a `let` annotated with a type parameter
//! other than the context.
//!
//! `#[uses]` only adds bounds on the context, so declaring the trait would not fix this: the provider
//! needs a bound on `Value`. rustc's `E0599` names `Value`, and the undeclared-trait detector leaves
//! it alone.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
fn person_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetValue<Value>)]
impl<Value: Default> Greeter {
    fn greet(&self) -> String {
        let made = Value::default();
        let value: &Value = &made;
        format!("Hello, {}!", value.person_name())
    }
}

fn main() {}
