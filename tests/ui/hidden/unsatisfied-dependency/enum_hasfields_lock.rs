//! A resolution-class `E0599` near CGP wiring, which must not crash the resolver.
//!
//! The `GreetChoice` provider's `where` clause names `Choice::Fields`, but `Choice` is an enum
//! with no such associated item reachable that way (the writer meant `<Choice as HasFields>::Fields`
//! and forgot the qualified form), so rustc reports `E0599: no variant named Fields`. Crucially that
//! error is emitted *during* predicate lowering (`gather_explicit_clauses_of`), while that query
//! is mid-flight.
//!
//! Running the trait solver on this `E0599` would re-force an emitting query and re-enter the
//! already-held `DiagCtxt` lock, aborting the compiler with `lock was already held`. The resolver
//! therefore declines an `E0599` that is not the "method exists but its trait bounds were not
//! satisfied" shape, which is both crash-safe (no solving on it) and correct, since a
//! name-resolution error is not a CGP wiring failure. rustc's own clear `E0599` passes through
//! unchanged.
//!
//! CGP error class: none; rustc's own name-resolution error, which the tool passes through.

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub enum Choice {
    Yes,
    No,
}

#[cgp_impl(new GreetChoice)]
impl Greeter
where
    Choice::Fields: Sized,
{
    fn greet(&self) {}
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetChoice,
    }
}

fn main() {}
