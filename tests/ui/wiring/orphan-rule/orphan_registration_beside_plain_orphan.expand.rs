#![feature(prelude_import)]
//! An orphan registration beside a plain Rust orphan impl, each with its own error.
//!
//! The registration keys `String` into a foreign component's per-type table, which is
//! `[CGP-E022]`, its attribute written by its full path. The `impl Display for Vec<u8>` beside it,
//! and the one a `macro_rules!` macro writes for `Vec<u16>`, are rustc's plain `E0117`s, which keep
//! rustc's wording, since the registration's fallback takes only a caret a CGP macro generated. The
//! program's macro has a CGP-like name to pin that the crate defining a macro decides, not its name.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;
impl<__Context__> ErrorRaiser<__Context__, String> for RaisePlain
where
    __Context__: HasErrorType,
{
    fn raise_error(error: String) -> <__Context__ as HasErrorType>::Error {
        {
            ::core::panicking::panic_fmt(format_args!("{0}", error));
        }
    }
}
impl<__Context__> IsProviderFor<ErrorRaiserComponent, __Context__, (String)>
for RaisePlain
where
    __Context__: HasErrorType,
{}
pub struct RaisePlain;
impl<__Components__> DefaultImpls1<ErrorRaiserComponent, __Components__> for String {
    type Delegate = RaisePlain;
}
impl std::fmt::Display for Vec<u8> {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
macro_rules! cgp_impl_display {
    ($ty:ty) => {
        impl std::fmt::Display for $ty { fn fmt(& self, _formatter : & mut
        std::fmt::Formatter <'_ >) -> std::fmt::Result { Ok(()) } }
    };
}
impl std::fmt::Display for Vec<u16> {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
fn main() {}
