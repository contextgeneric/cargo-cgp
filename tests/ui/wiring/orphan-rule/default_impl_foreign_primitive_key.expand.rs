#![feature(prelude_import)]
//! Orphan-rule violation: a crate registers a `#[default_impl]` for a primitive into a
//! per-type table of a foreign component. `DefaultImpls1<ErrorRaiserComponent>` and
//! `u64` are both foreign, so the generated impl has a foreign trait and a foreign
//! self type, with no local type covering `__Components__`, and the orphan rule
//! rejects it (`E0210`).
//!
//! cargo-cgp reshapes it into a `[CGP-E022]` header naming the key as a type. An
//! `E0210` carries no type in its message, so the compiler has not yet printed one
//! with trimmed paths; the detector must print the key by its own name, or the first
//! print takes the diagnostic context's lock the emitter holds. A primitive has no definition of its own, so it counts as foreign too. This is the
//! per-type sibling of `default_impl_foreign_component.rs`.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;
impl<__Context__> ErrorRaiser<__Context__, u64> for RaisePlain
where
    __Context__: HasErrorType,
{
    fn raise_error(error: u64) -> <__Context__ as HasErrorType>::Error {
        {
            ::core::panicking::panic_fmt(format_args!("{0}", error));
        }
    }
}
impl<__Context__> IsProviderFor<ErrorRaiserComponent, __Context__, (u64)> for RaisePlain
where
    __Context__: HasErrorType,
{}
pub struct RaisePlain;
impl<__Components__> DefaultImpls1<ErrorRaiserComponent, __Components__> for u64 {
    type Delegate = RaisePlain;
}
fn main() {}
