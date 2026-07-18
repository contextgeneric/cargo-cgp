//! Regression: a resolution-class `E0599` near CGP wiring must not crash the resolver.
//!
//! The `GreetChoice` provider's `where` clause names `Choice::Fields` — but `Choice` is an enum
//! with no such associated item reachable that way (the writer meant `<Choice as HasFields>::Fields`
//! and forgot the qualified form), so rustc reports `E0599: no variant named Fields`. Crucially that
//! error is emitted *during* predicate lowering (`gather_explicit_predicates_of`), while that query
//! is mid-flight.
//!
//! The resolver used to treat every `E0599` as a candidate consumer-method failure and run its trait
//! solver on this one; the solver re-forced an emitting query and re-entered the already-held
//! `DiagCtxt` lock, aborting the compiler with `lock was already held`. The resolver now declines an
//! `E0599` that is not the "method exists but its trait bounds were not satisfied" shape — which is
//! both crash-safe (no solving on it) and correct, since a name-resolution error is not a CGP wiring
//! failure. rustc's own clear `E0599` passes through unchanged.

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
