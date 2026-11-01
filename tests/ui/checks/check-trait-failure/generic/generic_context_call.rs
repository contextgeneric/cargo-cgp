//! A consumer called on one instantiation of a generic context, missing a field.
//!
//! The receiver names `App<u32>`, so the seed holds no type parameter and the walk resolves, though
//! the wiring is generic. The diagnostic's spans also land on the definition of `App<T>`, whose
//! type parameter makes the use-site anchor's own seed decline.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use core::marker::PhantomData;

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
pub struct App<T> {
    pub other: PhantomData<T>,
}

delegate_components! {
    <T> App<T> {
        GreeterComponent: GreetName,
    }
}

fn main() {
    let app: App<u32> = App { other: PhantomData };
    let _ = app.greet();
}
