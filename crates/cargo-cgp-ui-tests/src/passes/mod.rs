//! The verification passes and checks each fixture goes through: [`cgp_stderr_pass`],
//! [`rust_stderr_pass`], and [`expand_pass`] against their snapshots, [`json_pass`] for `//@json`,
//! and [`status_check`], [`expand_status_check`], [`pass_through_check`], and [`header_check`]. See
//! `cgp-knowledge-base/cargo-cgp/implementation/testing.md`.

mod checks;
mod codes;
mod pass_paths;
mod reduce_json;
mod run_passes;
mod snapshot_passes;

pub use checks::*;
pub use codes::*;
pub use pass_paths::*;
pub use reduce_json::*;
pub use run_passes::*;
pub use snapshot_passes::*;
