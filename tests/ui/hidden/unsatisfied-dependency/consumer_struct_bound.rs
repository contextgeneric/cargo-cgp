//! A consumer trait required by a struct's bound, built with a context whose wiring misses the field
//! the provider reads. The root cause is the missing `name` field on `App`, and `Holder` only passes
//! the requirement on.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}!")
    }
}

pub struct Holder<T: CanGreet>(pub T);

#[derive(HasField)]
pub struct App {
    pub label: String,
}

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    let app = App { label: String::new() };
    let _holder = Holder(app);
}
