//! What a resolved check failure's dependency chain bottoms out on.
//!
//! These are the rustc-free leaf types the driver's typed resolver produces and the
//! [wording](super::wording) turns into diagnostic text. They carry only owned `String`s,
//! no compiler handles, so they outlive the inference contexts the driver reads them from
//! and are exercised by plain unit tests.

mod field_issue;
mod fixes;
mod reconcile;
mod variants;

pub use field_issue::*;
pub use fixes::*;
pub use variants::*;
