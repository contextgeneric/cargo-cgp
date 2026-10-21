//! A consumer method called on an unbounded type parameter, where the consumer trait is not in
//! scope.
//!
//! `CanGreet` lives in the module `greeting`, which this module does not import, and
//! `fn run<Context>` calls `context.greet()`. The compiler reports no method named `greet` and
//! suggests bounding `Context` by `CanGreet` or by `Greeter</* __Context__ */>`, the provider trait
//! with the context parameter its macro generated, which no program can write. The tool keeps the
//! error as rustc wrote it, the wrong suggestion included. The call-site anchor does not read this
//! error, which the compiler raises before checking any bound.
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
