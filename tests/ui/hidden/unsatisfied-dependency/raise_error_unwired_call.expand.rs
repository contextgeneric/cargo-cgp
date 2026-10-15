#![feature(prelude_import)]
//! A generic function bounded on `CanRaiseError<String>`, called on a context with no raiser wired.
//!
//! `fail::<App>()` requires `App: CanRaiseError<String>`. The bound-call anchor reads the bound off
//! `fail`'s own `where` clauses, with `App` from the turbofish, and the walk reaches the missing
//! entry: ``[CGP-E001] missing wiring for `ErrorRaiserComponent` on `App` ``, with the wiring
//! `help`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;
pub struct App;
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
fn fail<Context: CanRaiseError<String>>() -> Context::Error {
    Context::raise_error(String::new())
}
fn main() {
    let _ = fail::<App>();
}
