//! A provider body calls another provider by its own path, `WidthArea::area(self)`, without declaring
//! the requirement that provider has of the context (`#[uses(HasWidth)]`).
//!
//! The tool leaves rustc's `E0277` on `__Context__: HasField<Symbol!("width")>`, which names the
//! generated context and the field the called provider reads rather than the declaration the
//! calling provider lacks. A concrete provider is not an inner provider, so `[CGP-E031]` does not
//! claim the call.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_auto_getter]
pub trait HasWidth {
    fn width(&self) -> f64;
}

#[cgp_impl(new WidthArea)]
#[uses(HasWidth)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        self.width()
    }
}

#[cgp_impl(new DoubledArea)]
impl AreaCalculator {
    fn area(&self) -> f64 {
        WidthArea::area(self) * 2.0
    }
}

fn main() {}
