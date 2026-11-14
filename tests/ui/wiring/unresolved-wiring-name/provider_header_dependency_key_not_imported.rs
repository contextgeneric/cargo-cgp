//! Provider headers that import a dependency's provider trait but not its key.
//!
//! `Waver` is defined in a module of the library, `Greeter` in a crate the library re-exports as
//! `core`, and `ErrorTypeProvider` in `cgp`. rustc's import search does not enter a re-exported
//! crate, so it suggests no import for the latter two, and a wrong type parameter on stable. The
//! tool reshapes each into `[CGP-E041]` with the key's public path from a direct dependency.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E041
//@aux-build: cgp-test-component-library

use cgp::core::error::ErrorTypeProvider;
use cgp::prelude::*;
use cgp_test_component_library::core::Greeter;
use cgp_test_component_library::direct::Waver;

#[cgp_impl(new WaveHand)]
impl Waver {
    fn wave(&self) -> String {
        "wave".to_owned()
    }
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

#[cgp_impl(new UseStringError)]
impl ErrorTypeProvider {
    type Error = String;
}

fn main() {}
