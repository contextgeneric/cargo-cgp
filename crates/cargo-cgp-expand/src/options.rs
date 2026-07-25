//! What to expand, and how much of the compiler's spelling to clean up besides resugaring.

use crate::select::ItemPath;

/// Options for one expansion.
#[derive(Clone, Debug)]
pub struct ExpandOptions {
    /// Narrow the expansion to one module or item, instead of showing the whole crate.
    ///
    /// `None` expands everything, which is the default and what a reader wants the first time. A
    /// path selects what [`select_items`](crate::select::select_items) matches — an item declared
    /// there, an impl written *for* that type, or an impl *of* that trait.
    pub item: Option<ItemPath>,
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
            item: None,
            strip_cgp_prefixes: true,
        }
    }
}
