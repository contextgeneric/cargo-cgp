//! A provider writing the abstract `Error` without its own `#[use_type(HasErrorType.Error)]`.
//!
//! The `E0425` keeps rustc's import suggestions (`std::io::Error` and others, all wrong) beside two
//! `E0277`s on `__Context__`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
