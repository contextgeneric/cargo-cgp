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
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;

#[cgp_impl(new RaiseString)]
#[default_impl(String in DefaultImpls1<ErrorRaiserComponent>)]
#[use_type(HasErrorType.Error)]
impl ErrorRaiser<String> {
    fn raise_error(error: String) -> Error {
        panic!("{error}")
    }
}

#[cgp_impl(new RaiseStr)]
#[default_impl(&'static str in DefaultImpls1<ErrorRaiserComponent>)]
#[use_type(HasErrorType.Error)]
impl ErrorRaiser<&'static str> {
    fn raise_error(error: &'static str) -> Error {
        panic!("{error}")
    }
}

fn main() {}
