use cgp::prelude::*;

// An ordinary Rust trait — *not* a CGP component (no `#[cgp_component]`, so no consumer/provider
// split and no `…Component` marker).
pub trait Greet {
    fn greet(&self);
}

// MISTAKE: `#[cgp_impl]` can only implement a CGP component's provider trait, but `Greet` is an
// ordinary trait. The macro turns the header inside out and references a `GreetComponent` marker
// that does not exist, producing the same cryptic cascade — but the fix differs from naming the
// wrong half of a component: `Greet` must become a component, or be implemented with a plain `impl`.
#[cgp_impl(new GreetHello)]
impl Greet {
    fn greet(&self) {
        println!("hello");
    }
}

fn main() {}
