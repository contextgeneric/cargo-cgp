//! An orphan registration beside `delegate_components!` on a foreign type, each with its own error.
//!
//! The registration keys `String` into a foreign component's per-type table, which is `[CGP-E022]`.
//! The wiring block's table is `Vec<u8>`, and a table is no namespace, though `DelegateComponent` has
//! a namespace's shape. Its impls are the program's own orphans, so they keep rustc's wording, and
//! they do not count as registrations that would make the caret ambiguous.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.

//@quality: usability
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

delegate_components! {
    Vec<u8> {
        ErrorRaiserComponent: RaisePlain,
    }
}

fn main() {}
