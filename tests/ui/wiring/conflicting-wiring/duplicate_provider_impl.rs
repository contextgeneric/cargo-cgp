//! Two `#[cgp_impl(GreetHello)]` blocks implement the same component for one provider struct, so
//! the provider-trait impls overlap (E0119). Unlike `duplicate_provider_name`, the struct is
//! declared once, so name resolution succeeds and coherence checking reports the overlap.
//!
//! rustc reports the overlap twice: on the provider trait `Greeter<_>`, and on the
//! `IsProviderFor<GreeterComponent, _>` impl each block also generates. The tool drops the second,
//! whose companion conflict it confirms, and keeps the first, whose carets cover each whole
//! `impl Greeter { … }` block.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct GreetHello;

#[cgp_impl(GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

// A second provider impl for the same provider and component.
#[cgp_impl(GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

fn main() {}
