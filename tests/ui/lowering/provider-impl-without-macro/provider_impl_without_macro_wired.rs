//! A provider-trait impl written without `#[cgp_provider]`, then wired and checked.
//!
//! The provider has no `IsProviderFor` impl, which the provider trait requires as a supertrait, so
//! the compiler rejects the impl on that bound and lists every unrelated implementor of
//! `IsProviderFor` beside it. The tool recognizes the impl structurally, words it as `[CGP-E039]`
//! with a `help` to add `#[cgp_provider]` or write the provider with `#[cgp_impl]`, and drops the
//! implementor list. The check that then fails on the provider's `IsProviderFor` bound restates the
//! same mistake, so the tool drops it.
//!
//! CGP error class: provider impl without the macro (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/provider-impl-without-macro.md).

//@quality: acceptable
//@codes: CGP-E039

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "hello".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
