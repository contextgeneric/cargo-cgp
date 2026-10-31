//! Wording a resolved check failure as diagnostic text.
//!
//! These are the pure `Resolved`-to-`String` builders the emitter's plan is composed from:
//! [`header`] words the class headline, [`headline`] the displayed headline that states the cause,
//! [`lead`] the one-sentence root-cause statements and their codes, [`note`] the `root cause:` note
//! bodies over their dependency chains, [`help`] the fixes each cause calls for, chosen in [`fixes`], and [`signature`]
//! the span-independent key that identifies one wiring failure across its many re-reports. Each is
//! a plain function over the rustc-free [`Resolved`](super::Resolved) model, so it is unit-tested
//! without a compiler; the [plan](super::plan) module composes them into the whole
//! [`DiagnosisPlan`](super::DiagnosisPlan).

mod fix_kind;
mod fixes;
mod header;
mod headline;
mod help;
mod lead;
mod mismatch_leaf;
mod note;
mod quoted_list;
mod required_type;
mod signature;

pub use header::*;
pub use headline::*;
pub use help::*;
pub use lead::*;
pub use mismatch_leaf::*;
pub use note::*;
pub use quoted_list::*;
pub use required_type::*;
pub use signature::*;
