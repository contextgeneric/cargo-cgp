//! Two `delegate_and_check_components!` tables for one context.
//!
//! Each table derives the check trait `__CanUseApp`, so the second definition is an `E0428`. The
//! tool reads it as two tables checking `App`, as it does for two `check_components!` tables, and
//! names the macro the program wrote.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: usability
//@codes: CGP-E016

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Namer)]
pub trait CanName {
    fn name(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

#[cgp_impl(new NameFixed)]
impl Namer {
    fn name(&self) -> String {
        "app".to_owned()
    }
}

pub struct App;

// MISTAKE: a second `delegate_and_check_components!` table for `App` in the same module derives the
// same check trait.
delegate_and_check_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

delegate_and_check_components! {
    App {
        NamerComponent: NameFixed,
    }
}

fn main() {}
