//! A provider writing the abstract `Error` without its own `#[use_type(HasErrorType.Error)]`.
//!
//! rustc reports three errors for the one missing import: an `E0425` on `Error` with import
//! suggestions (`std::io::Error` and others, all wrong), and two `E0277`s on `__Context__`, since the
//! component's `#[use_type]` requires `HasErrorType` of every context. The tool reports one
//! `[CGP-E035]` on `Error` with the `#[use_type(HasErrorType.Error)]` import in a `help`, and drops
//! the two `E0277`s, which the same import fixes.
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
    fn load(&self) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}
