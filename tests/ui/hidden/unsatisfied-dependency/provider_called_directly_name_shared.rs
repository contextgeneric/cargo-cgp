//! A provider's own method called through its path, where a provider trait of another module
//! declares a function of the same name.
//!
//! `GreetHello::run(&Person {})` calls the `Greeter` provider trait's `run`, and `Person` lacks the
//! `name` field `GreetHello` reads. The module `other` defines a `Runner` provider trait whose
//! function is also named `run`, but this module does not import it. The provider-call anchor takes
//! only a provider trait in scope at the call, as the compiler's own lookup does, so the block names
//! the missing field rather than `Runner`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E002 CGP-E102 CGP-E104 CGP-E110

pub mod other {
    use cgp::prelude::*;

    #[cgp_component(Runner)]
    pub trait CanRun {
        fn run(&self) -> u8;
    }
}

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn run(&self) -> String;
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn run(&self) -> String {
        self.name().to_owned()
    }
}

#[derive(HasField)]
pub struct Person {}

fn main() {
    let _ = GreetHello::run(&Person {});
}
