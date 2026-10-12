//! A per-entry generic that never reaches the key, in a `cgp_namespace!` entry.
//!
//! The entry lowers to an impl of the namespace's lookup trait for its key, which leaves `T`
//! unconstrained, so the compiler rejects it with `E0207` and suggests making the component
//! generic, which is not the fix.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct GreetWith<T>(pub PhantomData<T>);

#[cgp_provider]
impl<Context, T> Greeter<Context> for GreetWith<T> {
    fn greet(_context: &Context) {}
}

cgp_namespace! {
    new AppNamespace {
        <T> GreeterComponent: GreetWith<T>,
    }
}

fn main() {}
