//! Acceptable failure: a provider registers itself under a prefixed component's *bare
//! marker* in the very namespace the component's `#[prefix(...)]` registers into, so the
//! registration collides with the prefix's own redirect.
//!
//! `#[prefix(@app in DefaultNamespace)]` emits `impl<__Components__>
//! DefaultNamespace<__Components__> for GreeterComponent` whose `Delegate` is a
//! `RedirectLookup` down `@app.GreeterComponent`, and `#[default_impl(GreeterComponent in
//! DefaultNamespace)]` emits a second impl of that same trait for that same marker, so
//! coherence rejects the pair (`E0119`). Unlike namespace_inherited_unprefixed_key.rs, no
//! inheritance blanket is involved: both impls are concrete registrations for one key, and
//! the collision is direct.
//!
//! The mistake is the same one in both, though — a prefixed component is addressed by its
//! path — so the tool reads the `RedirectLookup` straight off the prefix entry's `Delegate`
//! and reports `[CGP-E007]`, naming `@app.GreeterComponent` as the key to register under.
//! Without that, the bare marker key leaves the conflict unclassified and rustc's raw
//! `conflicting implementations of trait \`DefaultNamespace<_>\`` is all the reader gets.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md,
//! cgp-knowledge-base/cgp/reference/attributes/default_impl.md (Known issues), and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E007).

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
#[default_impl(GreeterComponent in DefaultNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

fn main() {}
