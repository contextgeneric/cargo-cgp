//! The warning for a workspace wrapper the front-end replaces.

use std::ffi::OsStr;
use std::path::Path;

use crate::report::warning;

/// The environment variable through which cargo runs a wrapper for each workspace crate, and
/// through which the front-end installs its driver.
pub const WORKSPACE_WRAPPER_ENV: &str = "RUSTC_WORKSPACE_WRAPPER";

/// The warning to print when the front-end replaces the user's own [`WORKSPACE_WRAPPER_ENV`], or
/// `None` when there is nothing to replace: the variable is unset, empty, or already the driver.
///
/// cargo runs one workspace wrapper, so the driver takes the user's place for the run, as
/// `cargo clippy` does; the warning tells the user their wrapper did not run.
pub fn workspace_wrapper_warning(existing: Option<&OsStr>, driver: &Path) -> Option<String> {
    let existing = existing.filter(|value| !value.is_empty())?;
    if Path::new(existing) == driver {
        return None;
    }
    Some(warning(&format!(
        "{WORKSPACE_WRAPPER_ENV} is set to `{}`; cargo-cgp replaces it with its driver for this \
         run, so that wrapper does not run",
        existing.to_string_lossy()
    )))
}
