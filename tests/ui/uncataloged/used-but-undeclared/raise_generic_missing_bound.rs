//! A provider calling `Self::raise_error(…)` without declaring `CanRaiseError` in `#[uses]`.
//!
//! `Self` lowers to the provider impl's generated `__Context__` parameter, which the `[CGP-E016]`
//! inner-provider check skips, so the tool keeps rustc's `E0599`: no `raise_error` found for the
//! type parameter `__Context__`. It states nothing false, but it leaks `__Context__` and suggests a
//! bound inside `#[use_type]`; the fix is `#[uses(CanRaiseError<String>)]`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
