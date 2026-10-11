//! A provider calling `Self::raise_error(…)` without declaring `CanRaiseError` in `#[uses]`.
//!
//! The tool reports `[CGP-E016]` "the inner provider `__Context__` is used but not imported" with a
//! `#[use_provider(__Context__: ErrorRaiser)]` help. `__Context__` is the generated context, not an
//! inner provider; the fix is `#[uses(CanRaiseError<String>)]`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E016

use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<String, Error>;
}

#[cgp_impl(new LoadOrFail)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self) -> Result<String, Error> {
        Err(Self::raise_error("failed".to_owned()))
    }
}

fn main() {}
