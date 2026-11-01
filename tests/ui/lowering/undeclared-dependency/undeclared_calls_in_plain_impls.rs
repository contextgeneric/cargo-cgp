//! A trait or provider called in impls no CGP macro wrote, where `#[uses]` and `#[use_provider]`
//! cannot go.
//!
//! The impls here sit over a type parameter, as a `#[cgp_fn]` or `#[cgp_impl]` impl does, but they
//! are written by hand, or with `#[cgp_new_provider]`, so the fix is an ordinary bound on the
//! parameter. Each `E0599` keeps rustc's headline, its names resugared, rather than reading as
//! `[CGP-E030]` or `[CGP-E031]`, whose helps name attributes these impls do not take.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md>.

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
pub fn get_count(&self, #[implicit] count: &u64) -> u64 {
    *count
}

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> u64;
}

pub trait Describe {
    fn describe(&self) -> u64;
}

impl<T> Describe for T {
    fn describe(&self) -> u64 {
        self.get_count()
    }
}

pub trait Run<C> {
    fn run(context: &C) -> u64;
}

pub struct Runner;

impl<C> Run<C> for Runner {
    fn run(context: &C) -> u64 {
        context.get_count()
    }
}

pub struct Wrapper<T>(T);

pub trait Twice {
    fn twice(&self) -> f64;
}

impl<T> Twice for Wrapper<T> {
    fn twice(&self) -> f64 {
        T::area(&self.0) * 2.0
    }
}

#[cgp_new_provider]
impl<Context> Greeter<Context> for GreetHello {
    fn greet(context: &Context) -> u64 {
        context.get_count()
    }
}

fn main() {}
