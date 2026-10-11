#![feature(prelude_import)]
//! `raise_error` called by bare name where two traits provide it.
//!
//! rustc's `E0034` suggestions print `cgp::cgp_core::cgp_error::ErrorRaiser` under the tool and omit
//! the provider trait's generics.
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
