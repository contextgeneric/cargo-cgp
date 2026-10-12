//! A provider calling `Self::raise_error(LoadError)` without declaring `CanRaiseError` in `#[uses]`.
//!
//! The associated-function form of an undeclared trait: `Self` lowers to the provider impl's
//! generated `__Context__` parameter, so rustc reports `E0599`, no `raise_error` found for the type
//! parameter `__Context__`, and suggests a bound spliced into `#[use_type(…)]`. The argument is the
//! unit struct `LoadError`, so the dependency to declare is `CanRaiseError<LoadError>`.
//!
//! CGP error class: a used-but-undeclared dependency, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(Debug)]
pub struct LoadError;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<String, Error>;
}

#[cgp_impl(new LoadOrFail)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self) -> Result<String, Error> {
        Err(Self::raise_error(LoadError))
    }
}

fn main() {}
