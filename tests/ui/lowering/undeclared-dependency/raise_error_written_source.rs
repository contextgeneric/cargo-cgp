//! A provider calling `Self::raise_error(LoadError)` without declaring `CanRaiseError` in `#[uses]`.
//!
//! The associated-function form of an undeclared trait: `Self` lowers to the provider impl's
//! generated `__Context__` parameter, so rustc reports `E0599`, no `raise_error` found for the type
//! parameter `__Context__`, and suggests a bound spliced into `#[use_type(…)]`. The tool reshapes
//! it into `[CGP-E030]`, naming `CanRaiseError`, and the argument is the unit struct `LoadError`,
//! so the help names `#[uses(CanRaiseError<LoadError>)]`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E030

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
