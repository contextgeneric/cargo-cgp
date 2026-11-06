//! A wrapper impl generic over the context's parameter, on a context missing a field.
//!
//! The impl-site anchor reads the supertrait `App<T>: CanGreet`, which holds a type parameter, so
//! the resolver declines for the reason `generic_check_context` gives, and rustc's headline stays
//! over the `missing field` help.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        name.to_owned()
    }
}

#[derive(HasField)]
pub struct App<T> {
    pub other: PhantomData<T>,
}

delegate_components! {
    <T> App<T> {
        GreeterComponent: GreetName,
    }
}

pub trait CanGreetTwice: CanGreet {
    fn greet_twice(&self) -> String;
}

impl<T> CanGreetTwice for App<T> {
    fn greet_twice(&self) -> String {
        self.greet()
    }
}

fn main() {}
