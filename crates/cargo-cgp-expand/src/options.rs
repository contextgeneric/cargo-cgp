//! What to expand.

use crate::select::ItemPath;

/// Options for one expansion.
#[derive(Clone, Debug, Default)]
pub struct ExpandOptions {
    /// Narrow the expansion to one module or item, instead of showing the whole crate.
    ///
    /// `None` expands everything, which is the default and what a reader wants the first time. A
    /// path selects what [`select_items`](crate::select::select_items) matches: an item declared
    /// there, an impl written *for* that type, or an impl *of* that trait.
    pub item: Option<ItemPath>,
}
