//! Usability: a missing field surfaced as an encoded `Symbol` through the two-line
//! "similar impl" hint.
//!
//! `Rectangle` derives `HasField` but carries only `width`, so the one near-miss field
//! makes rustc report the unmet bound through its two-line shape — "the trait
//! `HasField<…height…>` is not implemented … but trait `HasField<…width…>` is
//! implemented for it". The missing field name is present but written as a nested
//! `Symbol<6, Chars<'h', …>>`, so this is a usability problem (decode it back to
//! `height`), the two-line-hint counterpart of the collapsed-list form in `base_area_2`.
//!
//! This fixture is also the regression guard for a defeated hidden-root-cause bug: the
//! two-line hint diffs the two `HasField` symbols and elides every generic argument they
//! share to `_`, which dropped the shared `'h'` out of *both* field names and left
//! `height` unreadable from the text. The driver's injected `--verbose` turns that
//! elision off (see docs/implementation/rustc-diagnostic-internals.md), so the symbols
//! now print in full — watch this snapshot for a `_` returning.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md.

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
