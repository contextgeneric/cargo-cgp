#![feature(prelude_import)]
//! `#[cgp_impl]` on an ordinary trait that is not a CGP component.
//!
//! `Greet` has no `#[cgp_component]`, so it has no provider trait and no `GreetComponent` marker.
//! `#[cgp_impl]` still turns the impl header inside out and names the missing marker, so rustc
//! reports an `E0107` arity error among unrelated codes. The tool reshapes it into `[CGP-E014]`,
//! with a help to make `Greet` a component or write a plain `impl`. It differs from naming the
//! wrong half of a component (`consumer_trait_in_provider_impl`) in its fix.
//!
//! CGP error class: the wrong half of a component, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Greet {
    fn greet(&self);
}
impl<__Context__> Greet<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) {
        {
            ::std::io::_print(format_args!("hello\n"));
        };
    }
}
impl<__Context__> IsProviderFor<GreetComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
fn main() {}
