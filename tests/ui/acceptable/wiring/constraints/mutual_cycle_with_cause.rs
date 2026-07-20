//! Candidate fixture (feasibility probe): a two-component wiring cycle — `ProviderA`
//! depends on `CanB`, `ProviderB` depends back on `CanA` — walked alongside a
//! genuinely missing field. The resolver's cycle guard must cut the `CanA → CanB →
//! CanA` loop while still reporting the missing `width` field down the other branch.

use cgp::prelude::*;

#[cgp_component(ProviderA)]
pub trait CanA {
    fn a(&self);
}

#[cgp_component(ProviderB)]
pub trait CanB {
    fn b(&self);
}

#[cgp_auto_getter]
pub trait HasWidth {
    fn width(&self) -> f64;
}

#[cgp_impl(new DoA)]
#[uses(CanB, HasWidth)]
impl ProviderA {
    fn a(&self) {
        let _ = self.width();
        self.b();
    }
}

#[cgp_impl(new DoB)]
#[uses(CanA)]
impl ProviderB {
    fn b(&self) {
        self.a();
    }
}

#[derive(HasField)]
pub struct App {}

delegate_components! {
    App {
        ProviderAComponent: DoA,
        ProviderBComponent: DoB,
    }
}

check_components! {
    App {
        ProviderAComponent,
    }
}

fn main() {}
