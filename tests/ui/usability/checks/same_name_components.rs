//! Full-path component resolution: two components in different modules share the marker name
//! `AreaCalculatorComponent`. The driver keys its component-name map by each marker's *full
//! path* (not its bare name), so the two markers occupy distinct entries instead of one
//! clobbering the other.
//!
//! `App` wires the `shapes_a` component and misses its field, so the recovered dependency tree
//! must name `shapes_a`'s consumer trait (`CanMeasureA`) — not `shapes_b`'s `CanMeasureB`. That
//! is the observable proof that the resolver looked the marker up by full path: with the old
//! bare-name key the two `AreaCalculatorComponent` entries collided and the resolved consumer
//! trait was whichever entry happened to win.
//!
//! See docs/implementation/typed-root-cause-resolution.md (component-name resolution).

use cgp::prelude::*;

pub mod shapes_a {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanMeasureA {
        fn measure(&self) -> f64;
    }

    #[cgp_auto_getter]
    pub trait HasWidthA {
        fn width_a(&self) -> f64;
    }

    #[cgp_impl(new MeasureA)]
    impl AreaCalculator
    where
        Self: HasWidthA,
    {
        fn measure(&self) -> f64 {
            self.width_a()
        }
    }
}

pub mod shapes_b {
    use cgp::prelude::*;

    #[cgp_component(AreaCalculator)]
    pub trait CanMeasureB {
        fn measure(&self) -> f64;
    }

    #[cgp_auto_getter]
    pub trait HasWidthB {
        fn width_b(&self) -> f64;
    }

    #[cgp_impl(new MeasureB)]
    impl AreaCalculator
    where
        Self: HasWidthB,
    {
        fn measure(&self) -> f64 {
            self.width_b()
        }
    }
}

// `App` lacks the `width_a` field the `shapes_a` component needs.
#[derive(HasField)]
pub struct App {
    pub other: f64,
}

delegate_components! {
    App {
        shapes_a::AreaCalculatorComponent: shapes_a::MeasureA,
    }
}

check_components! {
    App {
        shapes_a::AreaCalculatorComponent,
    }
}

fn main() {}
