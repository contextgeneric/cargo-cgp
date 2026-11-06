//! What a resolved check failure's dependency chain bottoms out on.
//!
//! These are the rustc-free leaf types the driver's typed resolver produces and the
//! [wording](super::wording) turns into diagnostic text. They carry only owned `String`s,
//! no compiler handles, so they outlive the inference contexts the driver reads them from
//! and are exercised by plain unit tests. The choice of a `DelegateComponent` leaf's shape is
//! decided here too, over facts the driver reads.

mod delegate_leaf_shape;
mod field_issue;
mod fixes;
mod reconcile;
mod variants;

pub use delegate_leaf_shape::*;
pub use field_issue::*;
pub use fixes::*;
pub use variants::*;
