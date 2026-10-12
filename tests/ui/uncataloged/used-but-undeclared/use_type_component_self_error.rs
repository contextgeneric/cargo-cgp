//! A provider for a component with `#[use_type(HasErrorType.Error)]` that writes `Self::Error`
//! without importing it.
//!
//! rustc reports three errors for the one missing `#[use_type(HasErrorType.Error)]`: an `E0220` on
//! `Self::Error`, and two `E0277`s on `__Context__: HasErrorType` at the method's signature and at
//! the `#[cgp_impl]` attribute, since the component requires the trait of every context. The tool
//! reports one `[CGP-E035]` with the import, and drops the two `E0277`s, which the same import fixes.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.

//@quality: acceptable
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
