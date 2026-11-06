//! A per-entry lifetime that only the provider uses: `<'a> GreeterComponent: GreetRef<'a>`. The
//! entry lowers to an impl whose associated `Delegate` names `'a`, so the lifetime is unconstrained
//! (E0207).
//!
//! The tool leaves rustc's error and its suggestion to add the lifetime to `Person`, which does not
//! compile. `[CGP-E019]` lists only type and const parameters, since a lifetime the key leaves out is
//! legal unless an associated type uses it; the shape is an open issue of the class.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct GreetRef<'a>(pub PhantomData<&'a ()>);

#[cgp_provider]
impl<'a, Context> Greeter<Context> for GreetRef<'a> {
    fn greet(_context: &Context) {}
}

pub struct Person;

delegate_components! {
    Person {
        <'a> GreeterComponent: GreetRef<'a>,
    }
}

fn main() {}
