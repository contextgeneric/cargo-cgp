#![feature(prelude_import)]
//! An orphan registration through `#[cgp_impl]` imported under another name.
//!
//! The registration is the same `[CGP-E022]` as `default_impl_foreign_type_key` writes, and its
//! caret is the attribute. A `use ... as` changes the path the attribute is written with, so
//! recognizing the macro has to read its definition rather than that path.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::{cgp_impl as register, *};
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
fn main() {}
