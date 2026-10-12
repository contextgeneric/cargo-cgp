#![feature(prelude_import)]
//! The wiring key `ErrorTypeProviderComponent` used without its import from `cgp::core::error`.
//!
//! rustc reports the `E0425` twice at the key, once with a suggestion to add a type parameter, and
//! suggests no import. The tool reshapes it into one `[CGP-E041]` with the import in a `help`. The
//! path is found by walking the public modules of the crate's dependencies, since the compiler's
//! path printing names the re-exported `cgp_core` crate by its crate name, a path that does not
//! compile.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct App;
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
fn main() {}
