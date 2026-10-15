//! A generic function calling a consumer method without the consumer bound.
//!
//! `context.greet()` on `context: &Context` in `fn run<Context>` needs `Context: CanGreet`, which
//! `run` does not declare. The call-site anchor reads the receiver's type parameter off its
//! declared type and states the missing bound:
//! ``[CGP-E001] the type parameter `Context` of `run` is not bounded by `CanGreet` ``, with a
//! `help` naming the bound to add; rustc's "use associated function syntax" advice is dropped.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E129

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

fn run<Context>(context: &Context) {
    context.greet();
}

fn main() {}
