#![feature(prelude_import)]
//! A generic function bounded on `CanRaiseError<String>`, called on a context with no raiser wired.
//!
//! A raw `E0277` listing nine `ErrorRaiser` implementors; the resolver declines and the missing
//! `ErrorRaiserComponent` entry is never stated.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
