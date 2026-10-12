//! A `#[check_providers]` check whose component shares its marker name with another component.
//!
//! `shapes_a` and `shapes_b` each define a component whose marker is named `MeasurerComponent`,
//! with the provider traits `MeasurerA` and `MeasurerB`. The check on `MeasureWidthA` fails for a
//! missing field. The provider-check anchor maps the entry's marker to its provider trait by
//! `DefId`, so the tree's provider step names `MeasurerA`, never the same-named marker's
//! `MeasurerB`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E002 CGP-E102 CGP-E104 CGP-E110

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
    #[check_providers(shapes_a::MeasureWidthA)]
    App {
        shapes_a::MeasurerComponent,
    }
}

fn main() {}
