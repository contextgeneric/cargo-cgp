//! The mirror of higher_order_inner_dependency.rs, where the
//! *outer* layer of the same higher-order provider carries the unmet dependency.
//! `ScaledArea<BaseArea>` is wired as there, but here `Rectangle` supplies `base_area`
//! and not `scale_factor`, so the *inner* `BaseArea` layer would succeed and the
//! *outer* `ScaledArea` layer fails on its own `Self: HasScaleFactor`.
//!
//! This fixture pins the contrast with the inner-failure case. Plain `cargo check` puts the
//! `unsatisfied trait bound introduced here` caret on `ScaledArea`'s own
//! `Self: HasScaleFactor` clause, and its `required for …` chain is *shorter*: it reaches
//! `ScaledArea<BaseArea>`'s `IsProviderFor` and stops, never descending into `BaseArea`,
//! because the outer layer fails before it ever delegates inward. The tool's tree shows the
//! same shape: `ScaledArea<BaseArea>` over the missing `scale_factor` field, with no
//! `BaseArea` entry. Read alongside higher_order_inner_dependency.rs, the pair shows that
//! the layer at fault is identified by which provider's `where` clause the caret sits on and
//! how deep the chain runs. This is the check doing its job, not a macro defect.
//!
//! See cgp-knowledge-base/cgp/errors/checks/higher-order-provider-layer.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasBaseArea {
    fn base_area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasScaleFactor {
    fn scale_factor(&self) -> f64;
}

#[cgp_impl(new BaseArea)]
impl AreaCalculator
where
    Self: HasBaseArea,
{
    fn area(&self) -> f64 {
        self.base_area()
    }
}

#[cgp_impl(new ScaledArea<Inner>)]
#[use_provider(Inner: AreaCalculator)]
impl<Inner> AreaCalculator
where
    Self: HasScaleFactor,
{
    fn area(&self) -> f64 {
        self.scale_factor() * Inner::area(self)
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub base_area: f64,
    // missing `scale_factor`, so the outer `ScaledArea` layer fails
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: ScaledArea<BaseArea>,
    }
}

// Fails in the outer `ScaledArea` layer: `Rectangle` has `base_area` but not
// `scale_factor`, so the inner `BaseArea` dependency holds and `ScaledArea`'s does
// not.
check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}

fn main() {}
