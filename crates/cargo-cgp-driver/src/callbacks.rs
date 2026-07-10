//! The compiler callbacks the driver installs.

use rustc_interface::interface::Config;

use crate::emitter;

/// The driver's [`rustc_driver::Callbacks`] implementation.
///
/// Its `config` hook installs the diagnostic-rewriting [`emitter`], which replaces the
/// session's JSON emitter with one that names the consumer and provider traits behind a CGP
/// component marker in the compiler's wiring notes. Everything else about the compilation is
/// unchanged from plain `rustc`.
pub struct CgpCallbacks;

impl rustc_driver::Callbacks for CgpCallbacks {
    fn config(&mut self, config: &mut Config) {
        emitter::install(config);
    }
}
