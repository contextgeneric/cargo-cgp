//! Orphan-rule violation: a crate registers a `#[default_impl]` for a foreign struct into a
//! per-type table of a foreign component. `DefaultImpls1<ErrorRaiserComponent>` and
//! `String` are both foreign, so the generated impl has a foreign trait and a foreign
//! self type, with no local type covering `__Components__`, and the orphan rule
//! rejects it (`E0210`).
//!
//! cargo-cgp reshapes it into a `[CGP-E022]` header naming the key as a type. An
//! `E0210` carries no type in its message, so the compiler has not yet printed one
//! with trimmed paths; the detector must print the key by its own name, or the first
//! print takes the diagnostic context's lock the emitter holds. A struct key takes that path. This is the
//! per-type sibling of `default_impl_foreign_component.rs`.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.

//@quality: acceptable
//@codes: CGP-E022

use cgp::core::component::DefaultImpls1;
use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
use cgp::prelude::*;

#[cgp_impl(new RaisePlain)]
#[default_impl(String in DefaultImpls1<ErrorRaiserComponent>)]
#[use_type(HasErrorType.Error)]
impl ErrorRaiser<String> {
    fn raise_error(error: String) -> Error {
        panic!("{error}")
    }
}

fn main() {}
