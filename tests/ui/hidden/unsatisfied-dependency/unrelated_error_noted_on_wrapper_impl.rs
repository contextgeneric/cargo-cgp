//! A plain Rust error whose note points at a hand-written impl whose CGP supertrait also fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so the supertrait `CanGreet` of the
//! wrapper `CanGreetWith<T>` fails at its impl. A separate mistake, `needs::<Person>()`, asks for
//! `CanGreetWith<Rc<u8>>`, which the impl gives only for a `T: Send`. rustc aims that error at the
//! call and adds a note on the impl's header. The tool reports both errors, as plain `cargo check`
//! does: the `[CGP-E003]` block on the impl, and the `Send` error as rustc wrote it. The impl-site
//! anchor reads only the primary spans, so a note on the impl does not tie the `Send` error to it.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E003 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use std::rc::Rc;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        self.name().to_owned()
    }
}

#[derive(HasField)]
pub struct Person {}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

pub trait CanGreetWith<T>: CanGreet {
    fn greet_with(&self, value: T) -> String;
}

impl<T: Send> CanGreetWith<T> for Person {
    fn greet_with(&self, _value: T) -> String {
        self.greet()
    }
}

fn needs<Context: CanGreetWith<Rc<u8>>>() {}

fn main() {
    needs::<Person>();
}
