//! A plain Rust error whose note points at a wrapper impl on a foreign type whose chain reaches a
//! failing CGP consumer.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `Person: CanGreet` fails in the
//! `where` clause of `impl HasInner for Box<Person>`, which the wrapper `CanGreetVia<T>` has as a
//! supertrait. A separate mistake, `needs::<Box<Person>>()`, asks for `CanGreetVia<Rc<u8>>`, which
//! the impl on `Box<Person>` gives only for a `T: Send`. rustc aims that error at the call and adds
//! a note on the impl's header. The tool reports both errors: the CGP block, and the `Send` error as
//! rustc wrote it. The wrapper-chain anchor reads only the primary spans, so a note on the impl does
//! not tie the `Send` error to it.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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

pub trait HasInner {
    fn inner_greeting(&self) -> String;
}

impl HasInner for Box<Person>
where
    Person: CanGreet,
{
    fn inner_greeting(&self) -> String {
        self.as_ref().greet()
    }
}

pub trait CanGreetVia<T>: HasInner {
    fn greet_via(&self, value: T) -> String;
}

impl<T: Send> CanGreetVia<T> for Box<Person> {
    fn greet_via(&self, _value: T) -> String {
        self.inner_greeting()
    }
}

fn needs<Context: CanGreetVia<Rc<u8>>>() {}

fn main() {
    needs::<Box<Person>>();
}
