//! Usability: a provider with two independent unmet field dependencies, which should render as
//! two parallel branches in the dependency tree rather than a single spine.
//!
//! `GreetFullName` needs both `HasFirstName` and `HasLastName`, and `Person` supplies neither
//! field, so the one check failure has two distinct root causes. The dependency note should
//! branch at the provider into a `first_name` path and a `last_name` path.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasFirstName {
    fn first_name(&self) -> &str;
}

#[cgp_auto_getter]
pub trait HasLastName {
    fn last_name(&self) -> &str;
}

#[cgp_impl(new GreetFullName)]
impl Greeter
where
    Self: HasFirstName + HasLastName,
{
    fn greet(&self) {
        println!("Hello, {} {}!", self.first_name(), self.last_name());
    }
}

#[derive(HasField)]
pub struct Person {
    // missing both first_name and last_name fields to trigger two parallel branches
}

delegate_components! {
    Person {
        GreeterComponent: GreetFullName,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {}
