//! Acceptable failure: a missing entry in an aggregate provider that joins a namespace. The context
//! `App` joins `DefaultNamespace` and forwards the whole `@app` path to the aggregate provider
//! `AppComponents`, which is keyed by paths and so joins `DefaultNamespace` itself. `AppComponents`
//! binds `@app.GreeterComponent` but not `@app.NamerComponent`, so the check on `NamerComponent`
//! bottoms out on the aggregate's own namespace lookup,
//! `@app.NamerComponent: DefaultNamespace<AppComponents>`.
//!
//! The tree names the table each lookup runs in: the context's namespace hop reads `in App`, the
//! aggregate's reads `in AppComponents`, and the leaf is the `[CGP-E110]` provider-table leaf naming
//! `AppComponents`, so the reader adds the entry to the aggregate rather than to the context.
//!
//! See cgp-knowledge-base/cgp/errors/checks/unregistered-namespace-path.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E104, CGP-E110).

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Namer)]
#[prefix(@app in DefaultNamespace)]
pub trait CanName {
    fn name(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

// The aggregate is keyed by paths, so it joins the namespace to map a bare lookup to its path.
delegate_components! {
    new AppComponents {
        namespace DefaultNamespace;

        @app.GreeterComponent: GreetHello,
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @app: AppComponents,
    }
}

check_components! {
    App {
        NamerComponent,
    }
}

fn main() {}
