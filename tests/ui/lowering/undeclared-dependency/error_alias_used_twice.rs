//! A provider writing the abstract `Error` twice without its own `#[use_type(HasErrorType.Error)]`,
//! beside a function outside the provider that names an undeclared `Error` too.
//!
//! rustc reports an `E0425` on each `Error`, and two `E0277`s on `__Context__`, since the
//! component's `#[use_type]` requires `HasErrorType` of every context. The tool reports one
//! `[CGP-E035]` on the provider's first `Error` and drops its second and the two `E0277`s, which the
//! one import fixes. The `Error` in `describe` is outside the provider, so its `E0425` stays as
//! rustc wrote it.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E035

use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self) -> Result<String, Error>;

    fn reload(&self) -> Result<String, Error>;
}

pub fn describe(_error: Error) {}

#[cgp_impl(new LoadNothing)]
impl Loader {
    fn load(&self) -> Result<String, Error> {
        Ok(String::new())
    }

    fn reload(&self) -> Result<String, Error> {
        Ok(String::new())
    }
}

fn main() {}
