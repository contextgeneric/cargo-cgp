//! A provider calling `Self::raise_error(…)` without declaring `CanRaiseError` in `#[uses]`.
//!
//! `Self` lowers to the provider impl's generated `__Context__` parameter, so rustc reports
//! `E0599`, no `raise_error` found for the type parameter `__Context__`, leaking the generated name
//! and suggesting a bound spliced into `#[use_type(…)]`. The tool reshapes it into `[CGP-E030]`,
//! naming `CanRaiseError`. The argument `"failed".to_owned()` does not write its type, so the help
//! names `#[uses(CanRaiseError<SourceError>)]` and asks for the type in place of `SourceError`.
//!
//! CGP error class: a used-but-undeclared dependency, not yet in the catalog
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/README.md#a-gap-the-used-but-undeclared-and-wrong-half-classes).

//@quality: acceptable
//@codes: CGP-E030

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
