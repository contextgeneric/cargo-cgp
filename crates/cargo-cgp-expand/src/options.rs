//! How much of the compiler's spelling to clean up besides resugaring.

/// Options for one expansion.
#[derive(Clone, Copy, Debug)]
pub struct ExpandOptions {
    /// Whether to drop the `cgp::macro_prelude::` qualifier the CGP macros emit in front of
    /// every construct they reference.
    ///
    /// On by default, because that qualifier is noise no programmer wrote and it is what the
    /// resugaring has to see past anyway. General module qualifiers are always kept, unlike in
    /// a diagnostic: in source they carry information a reader may want. Turning this off keeps
    /// the output compilable.
    pub strip_cgp_prefixes: bool,
}

impl Default for ExpandOptions {
    fn default() -> Self {
        Self {
            strip_cgp_prefixes: true,
        }
    }
}
