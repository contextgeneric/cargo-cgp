//! A hand-written `DelegateComponent` impl without the forwarding `IsProviderFor`.
//!
//! A raw `E0599` leaking `IsProviderFor` and its source path in `cgp`; the resolver declines.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
