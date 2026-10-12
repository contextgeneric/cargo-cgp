//! `Self::Error` in a component's method signature without importing the abstract error type.
//!
//! The signature is copied into the consumer trait, where `Self` is the context, and into the
//! provider trait, where `#[cgp_component]` writes it as `__Context__`, so rustc reports the same
//! `E0220` twice at one caret, once for `Self` and once for `__Context__`. The fix is
//! `#[use_type(HasErrorType.Error)]` on the component.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Loader)]
pub trait CanLoad {
    fn load(&self) -> Result<String, Self::Error>;
}

fn main() {}
