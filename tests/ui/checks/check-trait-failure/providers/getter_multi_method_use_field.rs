//! A multi-method `#[cgp_getter]` trait wired to `UseField`.
//!
//! `[CGP-E115]` is accurate but does not say that a multi-method getter has no `UseField` impl.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

use cgp::prelude::*;

#[cgp_getter]
pub trait HasDimensions {
    fn width(&self) -> &f64;
    fn height(&self) -> &f64;
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

delegate_components! {
    Rectangle {
        DimensionsGetterComponent: UseField<Symbol!("width")>,
    }
}

check_components! {
    Rectangle {
        DimensionsGetterComponent,
    }
}

fn main() {}
