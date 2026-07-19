//! A `check_components!` failure whose root cause is a plain type wired where a *provider* was
//! expected: a higher-order provider's inner slot is filled with a struct that does not implement
//! the provider trait at all. Distilled from the `money-transfer-api` example, where an endpoint
//! wrapper `UseBasicAuth<QueryBalanceRequest>` is missing its inner `HandleQueryBalance<…>` handler,
//! so the *request* type `QueryBalanceRequest` sits where an `ApiHandler` provider belongs.
//!
//! Here `WrapGreeter<Inner>` requires `Inner: Greeter` (its inner provider), but the context wires
//! `WrapGreeter<NotAGreeter>` where `NotAGreeter` is an ordinary struct with no `Greeter` impl. The
//! walk reaches `NotAGreeter: Greeter<App>`, whose only matching impl is the CGP delegation blanket,
//! so it bottoms out on an unmet `NotAGreeter: DelegateComponent<GreeterComponent>`.
//!
//! The resolver tells this apart from a leaf-provider dead-end (a valid provider reached by an input
//! mismatch, whose real cause runs through its concrete impl) by whether the owner has a concrete
//! impl of the provider trait at all: `NotAGreeter` has *no* `Greeter` impl, so it is genuinely not
//! a provider, reported as a [`CGP-E111`] `NotAProvider` leaf — `the provider trait \`Greeter\` is
//! not implemented for \`NotAGreeter\``. Before this, the resolver dropped the leaf and declined to a
//! `[CGP-E002]` block naming the whole `WrapGreeter<NotAGreeter>` pipeline (and leaking rustc's giant
//! implementor list), with the real cause nowhere.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

/// A real leaf provider for `Greeter`.
#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

/// A higher-order provider that wraps an inner `Greeter` — the shape a wrapper endpoint has.
#[cgp_impl(new WrapGreeter<Inner>)]
#[use_provider(Inner: Greeter)]
impl<Inner> Greeter {
    fn greet(&self) -> String {
        Inner::greet(self)
    }
}

/// An ordinary struct that is **not** a `Greeter` provider — wired where a provider is expected.
pub struct NotAGreeter;

pub struct App;

delegate_components! {
    App {
        GreeterComponent: WrapGreeter<NotAGreeter>,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
