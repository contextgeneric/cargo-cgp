//! Comparing a fixture's output against its committed snapshot, or blessing it.

mod diff;
mod outcome;
mod review;
mod snapshot_paths;

pub use diff::*;
pub use outcome::*;
pub use review::*;
pub use snapshot_paths::*;
