//! A wiring key guessed from the trait name, `HasScalarTypeComponent`.
//!
//! Two identical `E0425`s; the real key, `ScalarTypeProviderComponent`, is not suggested.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
