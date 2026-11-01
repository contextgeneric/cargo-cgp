//! A `#[cgp_impl]` provider in the explicit-context form calling a trait it does not declare.
//!
//! `impl<Context> Greeter for Context` names its own context, so the impl declares no reserved
//! `__Context__`, but `#[cgp_impl]` lowers it all the same and takes `#[uses]`. It is the macro's
//! own impl, which `undeclared_calls_in_plain_impls` tells apart from a hand-written one.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md>.

//@quality: acceptable
//@codes: CGP-E030

use cgp::prelude::*;

#[cgp_fn]
pub fn get_count(&self, #[implicit] count: &u64) -> u64 {
    *count
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> u64;
}

#[cgp_impl(new GreetExplicit)]
impl<Context> Greeter for Context {
    fn greet(&self) -> u64 {
        self.get_count()
    }
}

fn main() {}
