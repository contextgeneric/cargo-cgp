//! `#[cgp_impl]` on an ordinary trait that is not a CGP component.
//!
//! `Greet` has no `#[cgp_component]`, so it has no provider trait and no `GreetComponent` marker.
//! `#[cgp_impl]` still turns the impl header inside out and names the missing marker, so rustc
//! reports an `E0107` arity error among unrelated codes. The tool reshapes it into `[CGP-E033]`,
//! with a help to make `Greet` a component or write a plain `impl`. It differs from naming the
//! wrong half of a component (`consumer_trait_in_provider_impl`) in its fix.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E033

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
