//! Finding the leaf a mismatch headline states.

use crate::diagnosis::leaf::Leaf;
use crate::diagnosis::resolved::Resolved;

/// The first field-type-mismatch cause of a resolution, if any: the leaf `[CGP-E004]` is
/// worded from.
pub fn mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::FieldTypeMismatch { .. }))
}

/// The first associated-type-mismatch cause of a resolution, if any: the leaf `[CGP-E005]` is
/// worded from.
pub fn assoc_mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::AssocTypeMismatch { .. }))
}

/// The first variant-payload-mismatch cause of a resolution, if any: the extensible-data
/// counterpart of [`mismatch_leaf`], which an `E0271` block states as its headline.
pub fn variant_mismatch_leaf(resolved: &Resolved) -> Option<&Leaf> {
    resolved
        .causes
        .iter()
        .map(|cause| &cause.leaf)
        .find(|leaf| matches!(leaf, Leaf::VariantTypeMismatch { .. }))
}
