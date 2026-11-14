//! Upstream library for the UI fixtures whose provider headers name a dependency's provider trait.
//!
//! `Waver` is defined in this crate's own `direct` module, and `cgp-test-crate-a`, whose `Greeter`
//! component sits at its root, is re-exported whole as `core`. A downstream provider that imports
//! either provider trait without its key reaches the key only through these public paths:
//! `cgp_test_component_library::direct::WaverComponent` and
//! `cgp_test_component_library::core::GreeterComponent`.

pub use cgp_test_crate_a as core;

/// A component defined in this crate's own module.
pub mod direct {
    use cgp::prelude::*;

    /// Waves, as a component a downstream crate provides.
    #[cgp_component(Waver)]
    pub trait CanWave {
        fn wave(&self) -> String;
    }
}
