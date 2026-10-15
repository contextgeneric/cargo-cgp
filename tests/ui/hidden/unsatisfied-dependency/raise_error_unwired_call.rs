//! A generic function bounded on `CanRaiseError<String>`, called on a context with no raiser wired.
//!
//! `fail::<App>()` requires `App: CanRaiseError<String>`. The bound-call anchor reads the bound off
//! `fail`'s own `where` clauses, with `App` from the turbofish, and the walk reaches the missing
//! entry: ``[CGP-E001] missing wiring for `ErrorRaiserComponent` on `App` ``, with the wiring
//! `help`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E113

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}

fn fail<Context: CanRaiseError<String>>() -> Context::Error {
    Context::raise_error(String::new())
}

fn main() {
    let _ = fail::<App>();
}
