//! Running an update from start to finish.

use std::env;

use anyhow::Context;

use crate::config::{FRONTEND_CRATE, TOOL_VERSION};
use crate::report::status;
use crate::update::{
    fetch_sparse_index, parse_versions, reinstall_frontend, run_new_setup, select_update,
};

/// Find a newer in-channel version and, if there is one, reinstall the front-end and run
/// the new `setup`. Returns the exit code to propagate.
pub fn run_update() -> anyhow::Result<i32> {
    let index = fetch_sparse_index(FRONTEND_CRATE)?;
    let versions = parse_versions(&index);

    let Some(latest) = select_update(&versions, TOOL_VERSION)? else {
        eprintln!("note: cargo-cgp v{TOOL_VERSION} is already up to date");
        return Ok(0);
    };

    // Resolve our own path before the reinstall replaces the file. On Linux `current_exe` reads
    // `/proc/self/exe`, which names a replaced binary as `<path> (deleted)`, a path that does not
    // exist; read beforehand, it is the path the new binary is installed at.
    let exe = env::current_exe().context("failed to locate the cargo-cgp executable")?;

    eprintln!(
        "{}",
        status(
            "Updating",
            &format!("cargo-cgp v{TOOL_VERSION} to v{latest}")
        )
    );
    reinstall_frontend(&latest)?;

    // Hand off to the newly installed front-end's setup, which knows the new pinned
    // toolchain and provisions the matching driver.
    run_new_setup(&exe)
}
