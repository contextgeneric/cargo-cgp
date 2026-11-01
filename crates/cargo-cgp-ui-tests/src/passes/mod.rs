//! The verification passes and checks each fixture goes through.
//!
//! Three passes run per fixture and compare against a committed snapshot. Two are about what the
//! compiler *says*, and the third is about what the macros *generate*:
//!
//! - [`cgp_stderr_pass`] compares the tool's stderr to `.cgp.stderr`, the end-to-end check that
//!   the whole binary produces the expected output.
//! - [`rust_stderr_pass`] compares plain `cargo check`'s stderr to `.rust.stderr`, the "before"
//!   the tool improves on.
//! - [`expand_pass`] runs `cargo cgp expand` and compares its stdout to `.expand.rs`, the code the
//!   compiler was actually given, which is where the answer to "why does it say that?" usually is.
//!
//! Three checks read how the commands ended or compare them, rather than a snapshot:
//!
//! - [`status_check`]: the tool and plain `cargo check` must agree on whether the fixture
//!   compiles.
//! - [`expand_status_check`]: `cargo cgp expand` must fail exactly when it reported an error.
//! - [`pass_through_check`]: for a fixture under `pass-through/`, a plain Rust error the tool must
//!   not touch, the tool's output must equal plain rustc's, unless the fixture is marked
//!   `//@rewrites-falsely:` for a fix still to come or a documented limitation.
//!
//! [`header_check`] holds every error fixture to its structured header: the `//@quality:` it
//! declares, and the exact set of `[CGP-Exxx]` codes its `//@codes:` lists, checked against the
//! tool's output.
//!
//! A fixture marked `//@json` also runs [`json_pass`], which pins the diagnostics cargo's JSON
//! stream carries, the form an editor reads.

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
