//! A provider-trait impl written by hand beside an `IsProviderFor` impl that covers fewer contexts.
//!
//! `GreetHello` implements `Greeter` for every `Context` but `IsProviderFor` only for `App`, so the
//! compiler reports `GreetHello: IsProviderFor<GreeterComponent, Context>` unmet at the impl's `Self`
//! type. The provider has an `IsProviderFor` impl for the component, so this is no impl written
//! without `#[cgp_provider]`, and the tool does not report it as one: the error falls to the
//! `[CGP-E002]` text rewrite, which calls the impl's parameter `Context` a context, beside rustc's
//! suggestion of a `where` bound.
//!
//! CGP error class: provider impl without the macro (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/provider-impl-without-macro.md).

//@quality: usability
//@codes: CGP-E002

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "hello".into()
    }
}

pub struct App;

impl IsProviderFor<GreeterComponent, App, ()> for GreetHello {}

fn main() {}
