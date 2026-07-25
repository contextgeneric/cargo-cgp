//! A present-but-underived field beside a genuinely missing one on the same struct.
//!
//! `Shape` carries `width` but no `#[derive(HasField)]`, and does not carry `depth` at
//! all; the provider reads both. The two shortcomings are distinct fixes — add the
//! derive for `width`, add the field for `depth` — so they must stay *two* root causes:
//! the underived-field coalescing (which merges several underived fields on one struct
//! into a single add-the-derive cause, as in `base_area_2`) keys on a *group* of
//! present-but-underived fields and must leave a lone underived field, and the missing
//! field beside it, untouched. Pins that boundary.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md
//! (derive-missing and missing-field variants together).

use cgp::prelude::*;

#[cgp_component(VolumeCalculator)]
pub trait CanCalculateVolume {
    fn volume(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasShapeFields {
    fn width(&self) -> f64;

    fn depth(&self) -> f64;
}

#[cgp_impl(new ShapeVolume)]
impl VolumeCalculator
where
    Self: HasShapeFields,
{
    fn volume(&self) -> f64 {
        self.width() * self.width() * self.depth()
    }
}

// The derive is missing, and so is the `depth` field.
// #[derive(HasField)]
pub struct Shape {
    pub width: f64,
}

delegate_components! {
    Shape {
        VolumeCalculatorComponent:
            ShapeVolume,
    }
}

check_components! {
    Shape {
        VolumeCalculatorComponent,
    }
}

fn main() {}
