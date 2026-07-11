//! Full-path component resolution: two components in different modules are forced to share the
//! *same* marker name `MeasurerComponent` (via the `#[cgp_component { name, provider }]` override),
//! yet carry different consumer *and* provider trait names — `CanMeasureA`/`MeasurerA` in
//! `shapes_a`, `CanMeasureB`/`MeasurerB` in `shapes_b`. The driver keys its component-name map by
//! each marker's *full path*, not its bare name, so the two markers occupy separate entries.
//!
//! `App` wires and checks *both* components and misses both fields, producing two check failures.
//! Each recovered tree must name its own module's consumer and provider traits — the `shapes_a`
//! failure `CanMeasureA`/`MeasurerA`, the `shapes_b` failure `CanMeasureB`/`MeasurerB` — with no
//! cross-over. With the old bare-name key the two `MeasurerComponent` entries collided and both
//! failures resolved to whichever entry happened to win; the full-path key keeps them apart.
//!
//! See docs/implementation/typed-root-cause-resolution.md (component-name resolution).

use cgp::prelude::*;

pub mod shapes_a {
    use cgp::prelude::*;

    #[cgp_component {
        name: MeasurerComponent,
        provider: MeasurerA,
    }]
    pub trait CanMeasureA {
        fn measure(&self) -> f64;
    }

    #[cgp_auto_getter]
    pub trait HasWidthA {
        fn width_a(&self) -> f64;
    }

    #[cgp_impl(new MeasureWidthA: MeasurerComponent)]
    impl MeasurerA
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

    #[cgp_component {
        name: MeasurerComponent,
        provider: MeasurerB,
    }]
    pub trait CanMeasureB {
        fn measure(&self) -> f64;
    }

    #[cgp_auto_getter]
    pub trait HasWidthB {
        fn width_b(&self) -> f64;
    }

    #[cgp_impl(new MeasureWidthB: MeasurerComponent)]
    impl MeasurerB
    where
        Self: HasWidthB,
    {
        fn measure(&self) -> f64 {
            self.width_b()
        }
    }
}

// `App` lacks both the `width_a` and `width_b` fields the two components need.
#[derive(HasField)]
pub struct App {
    pub other: f64,
}

delegate_components! {
    App {
        shapes_a::MeasurerComponent: shapes_a::MeasureWidthA,
        shapes_b::MeasurerComponent: shapes_b::MeasureWidthB,
    }
}

check_components! {
    App {
        shapes_a::MeasurerComponent,
        shapes_b::MeasurerComponent,
    }
}

fn main() {}
