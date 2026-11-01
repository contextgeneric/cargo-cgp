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

//@quality: acceptable
//@codes: CGP-E022

use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;

#[cgp::prelude::cgp_impl(new RaisePlain)]
#[default_impl(String in DefaultImpls1<ErrorRaiserComponent>)]
#[use_type(HasErrorType.Error)]
impl ErrorRaiser<String> {
    fn raise_error(error: String) -> Error {
        panic!("{error}")
    }
}

impl std::fmt::Display for Vec<u8> {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

macro_rules! cgp_impl_display {
    ($ty:ty) => {
        impl std::fmt::Display for $ty {
            fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Ok(())
            }
        }
    };
}

cgp_impl_display!(Vec<u16>);

fn main() {}
