//! A key's qualified path segment reads by its own name in `[CGP-E019]`.
//!
//! The entry's key is written `std::io::Error`, and the message says `Error`. An `E0207` carries no
//! type in its message, so the detector prints the key's type by its own name, which avoids the
//! first trimmed-path print and the diagnostic lock that print takes. Beside a local `Error`, as
//! here, the name alone does not say which type the entry means, and the caret on the entry does.
//! The other wiring-key codes on such errors, `[CGP-E017]`, `[CGP-E018]`, and `[CGP-E022]`, print the
//! same way.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md>.

//@quality: usability
//@codes: CGP-E019

use cgp::prelude::*;

#[derive(Debug)]
pub struct Error;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct GreetWith<T>(pub T);

pub struct App;

delegate_components! {
    App {
        <T> @GreeterComponent.std::io::Error: GreetWith<T>,
        @GreeterComponent.Error: GreetWith<u8>,
    }
}

fn main() {}
