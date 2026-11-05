//! `UseDelegate` wired for a component with a lifetime parameter that lacks `#[derive_delegate]`.
//!
//! `[CGP-E115]` says `UseDelegate` is not a provider of the component, and its `help` names the
//! `open` statement with one path segment per type parameter of the component. The provider trait
//! takes the lifetime ahead of the context, so the help skips the context by type position and
//! names `Item` alone, never the generated `__Context__`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

use cgp::prelude::*;

#[cgp_component(Labeller)]
pub trait CanLabel<'a, Item: 'a> {
    fn label(&self, item: &'a Item) -> &'a str;
}

pub struct Square(pub String);

#[cgp_impl(new SquareLabel)]
impl<'a> Labeller<'a, Square> {
    fn label(&self, item: &'a Square) -> &'a str {
        &item.0
    }
}

pub struct App;

// MISTAKE: `UseDelegate` dispatches only a component with `#[derive_delegate]`.
delegate_components! {
    App {
        LabellerComponent: UseDelegate<new LabelTable {
            Square: SquareLabel,
        }>,
    }
}

check_components! {
    <'a> App {
        LabellerComponent: (Life<'a>, Square),
    }
}

fn main() {}
