//! A hand-written `DelegateComponent` impl without the forwarding `IsProviderFor`.
//!
//! A check of `GreeterComponent` on `App` passes, since it needs only `GreetHello`'s
//! `IsProviderFor`; the call fails, because the delegation blanket a consumer call routes through
//! also needs `App: IsProviderFor<GreeterComponent, App>`, which only `delegate_components!`
//! writes:
//! ``[CGP-E128] the wiring of `GreeterComponent` on `App` has no `IsProviderFor` impl``, with a
//! `help` naming the `delegate_components!` entry to write.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E128

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

pub struct App;

impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}

fn main() {
    App.greet();
}
