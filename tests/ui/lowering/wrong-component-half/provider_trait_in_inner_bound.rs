//! A higher-order provider bounds its inner provider by the provider trait without the context
//! argument, `where Inner: AreaCalculator`.
//!
//! The tool leaves rustc's `E0107` and drops the compiler's help to add the context parameter the
//! macro generated to the bound, a name no program can write. The output stays `usability`: it names
//! no fix, where `#[use_provider(Inner: AreaCalculator)]` is the idiomatic import. `[CGP-E037]`
//! covers a provider trait in `#[uses]` on the context, not a bound on another parameter.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: usability
//@codes: none
//@json

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new ScaledArea<Inner>)]
impl<Inner> AreaCalculator
where
    Inner: AreaCalculator,
{
    fn area(&self) -> f64 {
        Inner::area(self) * 2.0
    }
}

fn main() {}
