//! An orphan registration through `#[cgp_impl]` imported under another name.
//!
//! The registration is the same `[CGP-E022]` as `default_impl_foreign_type_key` writes, and its
//! caret is the attribute. A `use ... as` changes the path the attribute is written with, so
//! recognizing the macro has to read its definition rather than that path.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.

//@quality: acceptable
//@codes: CGP-E022

use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::{cgp_impl as register, *};

#[register(new RaisePlain)]
#[default_impl(String in DefaultImpls1<ErrorRaiserComponent>)]
#[use_type(HasErrorType.Error)]
impl ErrorRaiser<String> {
    fn raise_error(error: String) -> Error {
        panic!("{error}")
    }
}

fn main() {}
