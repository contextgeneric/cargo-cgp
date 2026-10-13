//! Clean compile: an `@`-path segment named like a primitive type but not one.
//!
//! `Path!` keeps only real primitive names (`u32`, `bool`, `str`, …) as types, so the `f` of
//! `@app.f` is a symbol, as the expansion's `Symbol!("f")` shows, and the redirect and its entry
//! meet.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".into()
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent => @app.f,
        @app.f: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
