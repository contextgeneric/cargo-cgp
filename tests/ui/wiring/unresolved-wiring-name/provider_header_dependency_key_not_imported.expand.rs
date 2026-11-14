#![feature(prelude_import)]
//! Provider headers that import a dependency's provider trait but not its key.
//!
//! `Waver` is defined in a module of the library, `Greeter` in a crate the library re-exports as
//! `core`, and `ErrorTypeProvider` in `cgp`. rustc's import search does not enter a re-exported
//! crate, so it suggests no import for the latter two, and a wrong type parameter on stable. The
//! tool reshapes each into `[CGP-E041]` with the key's public path from a direct dependency.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProvider;
use cgp::prelude::*;
use cgp_test_component_library::core::Greeter;
use cgp_test_component_library::direct::Waver;
impl<__Context__> Waver<__Context__> for WaveHand {
    fn wave(__context__: &__Context__) -> String {
        "wave".to_owned()
    }
}
impl<__Context__> IsProviderFor<WaverComponent, __Context__, ()> for WaveHand {}
///Provider `WaveHand` for [`Waver`].
pub struct WaveHand;
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "Hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
impl<__Context__> ErrorTypeProvider<__Context__> for UseStringError {
    type Error = String;
}
impl<__Context__> IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
for UseStringError {}
///Provider `UseStringError` for [`ErrorTypeProvider`].
pub struct UseStringError;
fn main() {}
