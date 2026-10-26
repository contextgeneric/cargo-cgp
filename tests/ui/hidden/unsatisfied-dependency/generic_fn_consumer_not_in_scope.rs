//! A consumer method called on an unbounded type parameter, where the consumer trait is not in
//! scope.
//!
//! `CanGreet` lives in the module `greeting`, which this module does not import, and
//! `fn run<Context>` calls `context.greet()`. The compiler reports no method named `greet` and
//! suggests bounding `Context` by `CanGreet` or by the provider trait `Greeter`, whose argument it
//! fills with the context parameter the macro generated, a name no program can write. The tool
//! keeps the error as rustc wrote it, the wrong suggestion included: the resolver sends a "no
//! method named" error only to the data-call anchor, so the call-site anchor does not read it.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: usability
//@codes: none

pub mod greeting {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }
}

fn run<Context>(context: &Context) {
    context.greet();
}

fn main() {}
