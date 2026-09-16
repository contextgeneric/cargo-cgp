//! Acceptable failure: a namespace inherits a parent and then binds a bare, unprefixed
//! component key that the parent already *redirects* — the namespace-level counterpart
//! of namespace_unprefixed_key.rs, where a *context* makes the same mistake.
//!
//! `#[prefix(@app in DefaultNamespace)]` registers `GreeterComponent` in
//! `DefaultNamespace` as a `RedirectLookup` down `@app.GreeterComponent`, and
//! `new AppNamespace: DefaultNamespace` emits the inheritance blanket impl
//! `impl<Table, Key, Value> AppNamespace<Table> for Key where Key: DefaultNamespace<…>`,
//! which forwards every key the parent answers — `GreeterComponent` among them.
//! `#[default_impl(GreeterComponent in AppNamespace)]` then emits a second impl
//! `impl<Table> AppNamespace<Table> for GreeterComponent`, and the two overlap, so
//! coherence rejects the pair (`E0119`, a *single* conflict on
//! `AppNamespace<_> for GreeterComponent`, since a namespace emits only its own
//! lookup-trait impl, not the context-side `DelegateComponent`/`IsProviderFor` pair).
//!
//! The key is the mistake: a prefixed component is addressed by its *path*, so the
//! registration must read `@app.GreeterComponent`. The tool recovers that target by
//! normalizing `<GreeterComponent as DefaultNamespace<AppNamespace's table>>::Delegate`
//! through the trait solver, exactly as it does for the context-level shape, so this
//! reads as a *redirect collision* (`[CGP-E007]`) naming the path to write instead.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E007).

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

cgp_namespace! {
    new AppNamespace: DefaultNamespace
}

#[cgp_impl(new GreetHello)]
#[default_impl(GreeterComponent in AppNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

fn main() {}
