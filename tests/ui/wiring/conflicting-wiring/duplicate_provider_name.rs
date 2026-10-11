//! Two `#[cgp_impl(new GreetHello)]` blocks each declare a
//! `pub struct GreetHello;`, so the name is defined twice (E0428). `#[cgp_impl]`
//! lowers each block independently and has no view of the other, so it correctly
//! defers the clash to the compiler, exactly as two hand-written definitions
//! would. rustc stops once name resolution finds a type defined twice, so the
//! E0428 is the only error: the two provider impls never reach coherence
//! checking, and there is no expansion to record.
//!
//! The E0428 carets fall on the `GreetHello` name inside `#[cgp_impl(new …)]`
//! rather than on the whole attribute, because the synthesized provider struct is
//! emitted with the struct ident's span (see
//! cgp-macro-core/src/types/empty_struct.rs). A regression that stamped the
//! struct with `call_site` would move the carets back onto the macro attribute.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

// Re-declaring the same provider name emits a conflicting struct + impls.
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

fn main() {}
