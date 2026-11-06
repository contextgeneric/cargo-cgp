//! The compiler callbacks the driver installs.

use rustc_driver::Compilation;
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::ty::TyCtxt;

use crate::emitter;
use crate::expand::{ExpandRequest, print_expansion};

/// The driver's [`rustc_driver::Callbacks`] implementation.
///
/// Its `config` hook installs the diagnostic-rewriting [`emitter`], which wraps the emitter the
/// compiler would have built for the active error format. Aside from the injected flags and that
/// emitter, a check is unchanged from plain `rustc`.
///
/// In a check, `after_expansion` reshapes the name-resolution errors of the crate's wiring entries
/// ([`emitter::reshape_name_resolution_errors`]) and of a misspelled `#[use_type]` import
/// ([`emitter::reshape_use_type_misspellings`]), and drops the suggestions that name a generated
/// identifier ([`emitter::drop_generated_name_suggestions`]). When the front-end asked for an
/// expansion, it prints the expanded crate and stops the compilation there instead; see
/// [`crate::expand`].
pub struct CgpCallbacks {
    /// Set when the invocation carried the expand-mode flag, holding where to write the result.
    pub expand: Option<ExpandRequest>,
}

impl rustc_driver::Callbacks for CgpCallbacks {
    fn config(&mut self, config: &mut Config) {
        // Print a path in its shortest unambiguous form (`Option<String>` rather than
        // `std::option::Option<std::string::String>`), as rustc's own driver does; a compiler
        // embedded through `rustc_driver` otherwise prints full paths.
        config.opts.trimmed_def_paths = true;
        emitter::install(config);
    }

    fn after_expansion(&mut self, compiler: &Compiler, tcx: TyCtxt<'_>) -> Compilation {
        let Some(request) = &self.expand else {
            // Name resolution is over, so the wiring entries its errors sit in can be read back.
            emitter::reshape_name_resolution_errors(tcx);
            emitter::reshape_use_type_misspellings(tcx);
            emitter::drop_generated_name_suggestions(tcx);
            return Compilation::Continue;
        };

        print_expansion(&compiler.sess, tcx, request);

        // Nothing downstream is wanted: analyzing a crate we are only reading would cost a full
        // type-check for no output.
        Compilation::Stop
    }
}
