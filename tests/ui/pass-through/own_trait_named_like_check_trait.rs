//! The program's own traits named like the check traits CGP's check macros derive.
//!
//! `__CheckApp` and `__CanUseApp` are the names `check_components!` and
//! `delegate_and_check_components!` derive for `App`, and the program declares traits of those
//! names first, so each table's check trait is an `E0428` against the program's own trait. Only one
//! table checks `App` in each case, so this is no pair of tables sharing a check-trait name, and the
//! errors must stay exactly as rustc prints them.
//!
//! CGP error class: none; plain Rust the tool passes through.

//@rewrites-falsely: the check-trait detector reads only the second definition's span, so a table colliding with the program's own trait reads as two tables

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

// MISTAKE: the program declares the names the check macros derive for `App`.
pub trait __CheckApp {}

pub trait __CanUseApp {}

delegate_and_check_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
