#![feature(prelude_import)]
//! Two orphan registrations in one crate keep rustc's `E0210`.
//!
//! A `#[default_impl]`'s caret is the `#[cgp_impl]` attribute while its generated impl is spanned on
//! the key, so a caret matches no candidate by its range. One candidate is still unambiguous, which
//! gives `[CGP-E022]`, but two leave the tool unable to say which registration a caret means, and it
//! declines both. Each message then names `__Components__`, and each caret still points at the
//! attribute to change.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;
impl<__Context__> ErrorRaiser<__Context__, String> for RaiseString
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
for RaiseString
where
    __Context__: HasErrorType,
{}
pub struct RaiseString;
impl<__Components__> DefaultImpls1<ErrorRaiserComponent, __Components__> for String {
    type Delegate = RaiseString;
}
impl<__Context__> ErrorRaiser<__Context__, &'static str> for RaiseStr
where
    __Context__: HasErrorType,
{
    fn raise_error(error: &'static str) -> <__Context__ as HasErrorType>::Error {
        {
            ::core::panicking::panic_fmt(format_args!("{0}", error));
        }
    }
}
impl<__Context__> IsProviderFor<ErrorRaiserComponent, __Context__, (&'static str)>
for RaiseStr
where
    __Context__: HasErrorType,
{}
pub struct RaiseStr;
impl<__Components__> DefaultImpls1<ErrorRaiserComponent, __Components__>
for &'static str {
    type Delegate = RaiseStr;
}
fn main() {}
