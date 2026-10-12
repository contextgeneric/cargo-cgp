//! `UseField` wired to a `#[cgp_type]` component.
//!
//! `[CGP-E115]` is accurate; the fix, `WithField`, is not hinted.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E002 CGP-E101 CGP-E102 CGP-E115

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
}

delegate_components! {
    Rectangle {
        ScalarTypeProviderComponent: UseField<Symbol!("width")>,
    }
}

check_components! {
    Rectangle {
        ScalarTypeProviderComponent,
    }
}

fn main() {}
