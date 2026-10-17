//! `UseField` wired to a `#[cgp_type]` component.
//!
//! `[CGP-E115]` says `UseField` is not a provider of the type component, and its `help` names the
//! adapter that makes it one, `WithProvider<UseField<…>>` (the expansion of `WithField<…>`), which
//! the solver confirms.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

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
