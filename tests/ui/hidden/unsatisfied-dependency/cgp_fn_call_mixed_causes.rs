//! A direct call to a `#[cgp_fn]` trait method on a type without wiring, whose blanket impl fails
//! for two kinds of cause: `Describe` requires `Self: Clone`, which `Person` is not, and through
//! `GetName` a `name` field, which `Person` lacks.
//!
//! The call-site anchor resolves the call to the blanket trait, and since the causes are mixed, the
//! headline is the class sentence with a count. `Person` carries no wiring, so the sentence names it
//! plainly, `the trait `Describe` is not implemented for `Person`: 2 root causes`, not as a context.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E003 CGP-E104 CGP-E110 CGP-E117

use cgp::prelude::*;

#[cgp_fn]
pub fn get_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_fn]
#[uses(GetName)]
pub fn describe(&self) -> String
where
    Self: Clone,
{
    self.get_name()
}

#[derive(HasField)]
pub struct Person {
    pub age: u8,
}

fn describe_person(person: &Person) -> String {
    person.describe()
}

fn main() {}
