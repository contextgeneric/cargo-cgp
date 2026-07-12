//! Usability: a missing field read *directly* by a provider — the shorter-chain counterpart of
//! [`base_area_1`](base_area_1.rs).
//!
//! Where `base_area_1` reads its fields through a `#[cgp_auto_getter]` getter trait
//! (`HasRectangleFields`), `RectangleArea` here takes the `width`/`height` values as `#[implicit]`
//! arguments, so the unmet `HasField<Symbol!("height")>` bound sits *directly* on the provider's
//! `IsProviderFor` rather than one hop behind a getter trait. The resolved dependency chain is thus
//! one node shorter — `CanCalculateArea → AreaCalculator/RectangleArea → HasField height`, with no
//! intervening `trait impl` for the getter — which is exactly the distinction this fixture pins
//! against `base_area_1`.
//!
//! `Rectangle` derives `HasField` but carries only `width`, so the one near-miss field makes rustc
//! report the unmet bound through its two-line "similar impl" shape, with the missing field name
//! written as a nested `Symbol<6, Chars<'h', …>>`. The driver resolves it to the plain
//! `[CGP-E001]` "missing field `height`" form; the `--verbose` elision guard from `base_area_1`
//! still applies to the untransformed baseline.
//!
//! Exposes issues in docs/issues/usability.md. CGP error class:
//! ../../../../../cgp/docs/errors/checks/check-trait-failure.md.

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
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
