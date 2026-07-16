//! Acceptable failure: a context joins a namespace that *routes* a prefixed
//! component to a path, but no entry ever *terminates* that path with a provider,
//! so the namespace lookup finds no delegate.
//!
//! `CanGreet` carries `#[prefix(@app in DefaultNamespace)]`, so `DefaultNamespace`
//! resolves `GreeterComponent` to `RedirectLookup<_, @app.GreeterComponent>`. `App`
//! joins `DefaultNamespace` with `namespace DefaultNamespace;`, so its
//! `GreeterComponent` lookup follows that redirect — but nothing (no `#[default_impl]`,
//! no namespace body entry, no direct `@app.GreeterComponent:` line) ever binds a
//! provider at that path. The defined `GreetHello` is never wired there. The terminal
//! failure is the namespace lookup `Path!(@app.GreeterComponent): DefaultNamespace<App>`,
//! for which there is no impl.
//!
//! This is the *lookup-failed* class — no provider is found at all — distinct from
//! an unsatisfied *dependency*, where a provider is found but its `where` clause is
//! unmet. The forgotten binding (usually a missing `#[default_impl]` or body entry)
//! is the common namespace mistake it captures. The resolver recognizes the unmet
//! namespace-lookup trait by its `Delegate`-associated-type fingerprint and words the
//! root cause as a `MissingRedirectWiring`: the redirect forwards the lookup to the path
//! in `App`, but `App` has no delegate entry for it — naming the path the programmer must
//! wire rather than leaving a raw `DefaultNamespace` bound.
//!
//! See docs/errors/checks/unregistered-namespace-path.md.

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
