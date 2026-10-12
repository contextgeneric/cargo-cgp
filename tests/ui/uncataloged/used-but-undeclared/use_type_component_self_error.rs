//! A provider for a component with `#[use_type(HasErrorType.Error)]` that writes `Self::Error`
//! without importing it.
//!
//! The component requires `HasErrorType` of every context, and the provider names the error type
//! without the import, so rustc reports three errors for the one missing
//! `#[use_type(HasErrorType.Error)]`: an `E0220` on `Self::Error`, and two `E0277`s on
//! `__Context__: HasErrorType` at the method's signature and at the `#[cgp_impl]` attribute. The
//! tool reshapes the `E0220` into `[CGP-E035]` and keeps the two `E0277`s, which the same import
//! fixes.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E035

use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<String, Error>;
}

#[cgp_impl(new LoadNothing)]
impl Loader {
    fn load(&self) -> Result<String, Self::Error> {
        Ok(String::new())
    }
}

fn main() {}
