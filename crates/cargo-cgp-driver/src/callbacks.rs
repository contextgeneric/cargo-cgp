//! The compiler callbacks the driver installs.

use rustc_driver::Compilation;
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::ty::TyCtxt;

use crate::emitter;
use crate::expand::{ExpandRequest, print_expansion};

/// The driver's [`rustc_driver::Callbacks`] implementation.
///
/// Its `config` hook installs the diagnostic-rewriting [`emitter`], which replaces the
/// session's JSON emitter with one that names the consumer and provider traits behind a CGP
/// component marker in the compiler's wiring notes. Everything else about a check is
/// unchanged from plain `rustc`.
///
/// When the front-end asked for an expansion, `after_expansion` prints the expanded crate and
/// stops the compilation there instead; see [`crate::expand`].
pub struct CgpCallbacks {
    /// Set when the invocation carried the expand-mode flag, holding where to write the result.
    pub expand: Option<ExpandRequest>,
}

impl rustc_driver::Callbacks for CgpCallbacks {
    fn config(&mut self, config: &mut Config) {
        emitter::install(config);
    }

    fn after_expansion(&mut self, compiler: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        let Some(request) = &self.expand else {
            return Compilation::Continue;
        };

        print_expansion(&compiler.sess, tcx, request);

        // Nothing downstream is wanted: analyzing a crate we are only reading would cost a full
        // type-check for no output.
        Compilation::Stop
    }
}
