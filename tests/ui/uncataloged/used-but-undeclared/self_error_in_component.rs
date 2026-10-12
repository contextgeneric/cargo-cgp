//! `Self::Error` in a component's method signature without importing the abstract error type.
//!
//! The signature is copied into the consumer trait, where `Self` is the context, and into the
//! provider trait, where `#[cgp_component]` writes it as `__Context__`, so rustc reports the same
//! `E0220` twice at one caret, once for `Self` and once for `__Context__`. The tool reports one
//! `[CGP-E035]` with the `#[use_type(HasErrorType.Error)]` import, and drops the copy at the same
//! caret.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.

//@quality: acceptable
//@codes: CGP-E035

use cgp::prelude::*;

#[cgp_component(Loader)]
pub trait CanLoad {
    fn load(&self) -> Result<String, Self::Error>;
}

fn main() {}
