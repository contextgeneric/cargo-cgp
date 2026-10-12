//! A wiring key guessed from the trait name, `HasScalarTypeComponent`.
//!
//! The component of `HasScalarType` is `ScalarTypeProviderComponent`, so the guessed key is an
//! `E0425` "cannot find type", reported twice. The tool reshapes it into one `[CGP-E042]` whose
//! `help` names the real component.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E042

use cgp::prelude::*;

#[cgp_type]
pub trait HasScalarType {
    type Scalar;
}

pub struct App;

delegate_components! {
    App {
        HasScalarTypeComponent: UseType<f64>,
    }
}

fn main() {}
