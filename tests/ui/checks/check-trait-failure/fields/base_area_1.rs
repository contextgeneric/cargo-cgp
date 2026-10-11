//! A missing field whose name rustc encodes as a nested `Symbol`, in the two-line
//! "similar impl" hint.
//!
//! `Rectangle` derives `HasField` but carries only `width`, so the one near-miss field
//! makes rustc report the unmet bound through its two-line shape: "the trait
//! `HasField<…height…>` is not implemented … but trait `HasField<…width…>` is
//! implemented for it", with the field name written as `Symbol<6, Chars<'h', …>>`. The tool
//! resolves it into a `[CGP-E001]` tree whose leaf names the missing `height`. It is the
//! two-line-hint counterpart of the collapsed-list form in `base_area_2`.
//!
//! The fixture also guards against a hidden root cause: the two-line hint diffs the two
//! `HasField` symbols and elides every generic argument they share to `_`, which would drop the
//! shared `'h'` out of both field names. The driver's injected `--verbose` turns that elision
//! off (see cgp-knowledge-base/cargo-cgp/implementation/rustc-diagnostic-internals.md), so a `_`
//! appearing in this snapshot is a regression.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.

//@json
//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E105 CGP-E106

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasRectangleFields {
    fn width(&self) -> f64;

    fn height(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator
where
    Self: HasRectangleFields,
{
    fn area(&self) -> f64 {
        self.width() * self.height()
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    // missing height field to trigger error
    // pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent:
            RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
