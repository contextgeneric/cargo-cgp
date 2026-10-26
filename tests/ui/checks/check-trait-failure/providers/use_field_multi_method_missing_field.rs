//! A getter with two methods wired to `UseField` on a context that lacks one of the getter's
//! fields.
//!
//! `UseField<Symbol!("width")>` reads one field, so it implements no getter with two methods, and
//! the check reports it as no provider of `DimensionsGetter`. `UseFields`, which reads each method's
//! field by the method's name, would not implement the getter either, since `Rectangle` has no
//! `height`, so no `help` names it.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
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
