//! A per-entry generic that only the entry's redirect uses: `<T> GreeterComponent => @app.T`. The
//! entry's `Delegate` is a `RedirectLookup` naming `T`, and nothing binds it (E0207).
//!
//! The tool reports `[CGP-E019]`, and since the parameter appears in the redirect rather than in a
//! provider, the help says to write a concrete type where the entry uses it, without naming the
//! generated `RedirectLookup`.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E019).

//@quality: acceptable
//@codes: CGP-E019

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct Person;

delegate_components! {
    Person {
        <T> GreeterComponent => @app.T,
    }
}

fn main() {}
