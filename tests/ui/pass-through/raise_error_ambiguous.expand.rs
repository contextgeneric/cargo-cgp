#![feature(prelude_import)]
//! `raise_error` called by bare name where two traits provide it.
//!
//! The error is plain Rust: the call is ambiguous (`E0034`). The tool must print what plain
//! `cargo check` prints; its module strip shortens the `cgp::prelude::CanRaiseError` path rustc
//! prints in a note.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
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
impl DelegateComponent<ErrorRaiserComponent> for App {
    type Delegate = RaiseFrom;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for App
where
    RaiseFrom: IsProviderFor<ErrorRaiserComponent, __Context__, __Params__>,
{}
fn main() {
    let _ = App::raise_error(String::new());
}
