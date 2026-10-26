//! A provider-trait impl a `macro_rules!` macro writes without `#[cgp_provider]`.
//!
//! The macro writes `impl<Context> Greeter<Context> for GreetHello` with no `IsProviderFor` impl, so
//! the compiler rejects it on the provider trait's supertrait. The `[CGP-E039]` detector takes only
//! an impl whose trait reference the program wrote, and this one comes from a macro expansion, so
//! the error falls to the text rewrite: a `[CGP-E002]` headline that calls the impl's parameter
//! `Context` a context, over rustc's list of `IsProviderFor` implementors cut to eight.
//!
//! CGP error class: provider impl without the macro (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/provider-impl-without-macro.md).

//@quality: usability
//@codes: CGP-E002

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

macro_rules! greeter_provider {
    ($provider:ident) => {
        pub struct $provider;

        impl<Context> Greeter<Context> for $provider {
            fn greet(_context: &Context) -> String {
                "hello".into()
            }
        }
    };
}

greeter_provider!(GreetHello);

fn main() {}
