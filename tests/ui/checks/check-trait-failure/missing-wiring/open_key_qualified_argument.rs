//! An `open` dispatch key whose value type takes a type two modules share a name for.
//!
//! rustc prints `a::Item` qualified, because `b::Item` shares its name. The missing key is
//! `@GreeterComponent.Wrap<a::Item>`: the segment's own head is a plain type, and only its argument
//! holds a path.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E103 CGP-E113

use cgp::prelude::*;

pub mod a {
    pub struct Item;
}

pub mod b {
    pub struct Item;
}

pub struct Wrap<T>(pub T);

#[cgp_component(Greeter)]
pub trait CanGreet<Input> {
    fn greet(&self, input: Input);
}

#[cgp_impl(new GreetB)]
impl Greeter<Wrap<b::Item>> {
    fn greet(&self, _input: Wrap<b::Item>) {}
}

pub struct App;

delegate_components! {
    App {
        open GreeterComponent;

        @GreeterComponent.Wrap<b::Item>: GreetB,
    }
}

check_components! {
    App {
        GreeterComponent: Wrap<a::Item>,
    }
}

fn main() {}
