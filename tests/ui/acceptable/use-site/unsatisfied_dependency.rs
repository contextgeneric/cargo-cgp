//! Usability: a hidden-class error whose cause cargo-cgp's next-solver already
//! surfaces, leaving only a verbose, misleading presentation.
//!
//! `GreetHello` needs `Self: HasName`, but `Person` has no `name` field, and the
//! failure is triggered by calling `greet` directly with no `check_components!`.
//! Under a plain `cargo check` this is the hidden class — a bare `E0599` "method not
//! found" that never names the cause. cargo-cgp injects `-Znext-solver`, so the
//! snapshot instead surfaces the unmet `HasField<…name…>` bound and even an "add
//! #[derive(HasField)]" hint: the root cause is recoverable. What remains is a
//! usability problem — the primary line still reads "method not found … use
//! associated function syntax instead", misleading for a wiring error, wrapped
//! around the real note.
//!
//! CGP error class: https://github.com/contextgeneric/cgp/blob/main/docs/errors/hidden/unsatisfied-dependency.md.
//! Exposes issues in docs/issues/usability.md.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

#[derive(HasField)]
pub struct Person {
    // No `name` field — `GreetHello`'s `Self: HasName` dependency cannot be met.
    pub age: u8,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    let person = Person { age: 42 };
    // Hidden-cause error: the compiler says `greet`'s bounds are unsatisfied without
    // naming the missing `name` field.
    person.greet();
}
