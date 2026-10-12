//! A per-entry generic that never reaches the key, in `delegate_and_check_components!`.
//!
//! The macro lowers the entry to a `DelegateComponent` impl, its `IsProviderFor` twin, and a check
//! impl, and each copies the entry's generic list, so the compiler reports the unconstrained `T`
//! three times at one caret.
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

pub struct Person;

delegate_and_check_components! {
    Person {
        <T> GreeterComponent: GreetWith<T>,
    }
}

fn main() {}
