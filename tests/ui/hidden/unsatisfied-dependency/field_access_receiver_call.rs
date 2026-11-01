//! A consumer method called on a field access, which only typeck could type. The call-site anchor
//! cannot read that receiver, but the diagnostic's note lands on the context's definition, so the
//! use-site anchor recovers it.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetName)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        name.to_owned()
    }
}

#[derive(HasField)]
pub struct App {
    pub other: String,
}

delegate_components! {
    App {
        GreeterComponent: GreetName,
    }
}

pub struct Holder {
    pub app: App,
}

fn main() {
    let holder = Holder { app: App { other: String::new() } };
    let _ = holder.app.greet();
}
