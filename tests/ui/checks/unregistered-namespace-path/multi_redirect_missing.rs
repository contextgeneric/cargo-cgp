//! A namespace redirect that hops through *several* layers before landing on a path
//! nothing terminates. `CanGreet` is prefixed into `MyNamespace` at `@start`, so its lookup first
//! redirects to `@start.GreeterComponent`; a `=>` entry redirects that to `@middle`, and another
//! redirects `@middle` to `@end` — but nothing binds a provider at `@end`. Each hop reads as its
//! own `redirect lookup to \`Path\` in \`App\`` entry in the dependency chain, and the terminal
//! states the missing delegate entry in the same form a plain missing wiring uses.
//!
//! This is the multi-layer counterpart of `unregistered_prefix_path`: it pins that a chain of
//! `RedirectLookup` hops is rendered as successive redirect entries rather than one opaque step.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E103 CGP-E113

use cgp::prelude::*;

cgp_namespace! {
    new MyNamespace {
        @start.GreeterComponent =>
            @middle,
        @middle =>
            @end,
    }
}

#[cgp_component(Greeter)]
#[prefix(@start in MyNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct App;

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
