//! A check that carries a generic parameter, on a context missing a field.
//!
//! `App<T>` wires `GreeterComponent`, and `GreetName` reads a `name` field the struct lacks. The
//! resolver solves under an empty parameter environment, where the solver does not prove a wiring
//! lookup that names the check's own `T`, so walking the entry would report the wired component as
//! missing. It declines instead, and the text rewrite keeps rustc's headline over the
//! `missing field` help.
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

check_components! {
    <T> App<T> {
        GreeterComponent,
    }
}

fn main() {}
