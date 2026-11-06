//! Locating the driver executable.

use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::config::DRIVER_ENV;

/// Where the front-end found the driver it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverOrigin {
    /// Named by the [`DRIVER_ENV`] override.
    Override,
    /// Beside the running `cargo-cgp`, where `cargo install` and `setup` put it.
    Sibling,
    /// On `PATH`, because none sits beside the running `cargo-cgp`.
    Path,
    /// Neither beside `cargo-cgp` nor on `PATH`; the bare name is left for the OS to fail on.
    Missing,
}

/// The driver executable the front-end runs, and where it was found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DriverLocation {
    /// The path to run: absolute when found, the bare executable name when
    /// [`DriverOrigin::Missing`].
    pub path: PathBuf,
    /// Where the path came from.
    pub origin: DriverOrigin,
}

impl DriverLocation {
    /// How a message names this driver: its path and where it came from, so a stale driver found on
    /// `PATH` is not mistaken for the one installed beside the front-end.
    pub fn describe(&self) -> String {
        let path = self.path.display();
        match self.origin {
            DriverOrigin::Override => format!("the cargo-cgp-driver at {path} (from {DRIVER_ENV})"),
            DriverOrigin::Sibling => format!("the cargo-cgp-driver at {path} (beside cargo-cgp)"),
            DriverOrigin::Path => format!(
                "the cargo-cgp-driver at {path} (found on PATH, since none is beside cargo-cgp)"
            ),
            DriverOrigin::Missing => {
                "the cargo-cgp-driver (none is beside cargo-cgp or on PATH)".to_owned()
            }
        }
    }
}

/// Resolve the `cargo-cgp-driver` executable.
///
/// An explicit [`DRIVER_ENV`] override wins, so a driver installed elsewhere can be named.
/// Otherwise, since cargo and rustup lay the driver next to the `cargo-cgp` binary that is running
/// (both end up in the same `target/<profile>` or `~/.cargo/bin` directory), we look for a sibling
/// of the current executable, and failing that on `PATH`.
///
/// `driver_bin` is the executable name ([`crate::config::DRIVER_BIN`]); the platform executable
/// suffix (e.g. `.exe`) is appended here.
pub fn driver_path(driver_bin: &str) -> anyhow::Result<DriverLocation> {
    let file_name = format!("{driver_bin}{}", env::consts::EXE_SUFFIX);
    if let Some(explicit) = env::var_os(DRIVER_ENV) {
        return Ok(locate_driver(Some(explicit), None, None, &file_name));
    }

    let current =
        env::current_exe().context("failed to locate the running cargo-cgp executable")?;
    let path_var = env::var_os("PATH");
    Ok(locate_driver(
        None,
        current.parent(),
        path_var.as_deref(),
        &file_name,
    ))
}

/// The decision behind [`driver_path`], over its inputs: the override, the front-end's directory,
/// and the `PATH` value. It reads the file system only to test whether a candidate file exists.
pub fn locate_driver(
    explicit: Option<OsString>,
    front_end_dir: Option<&Path>,
    path_var: Option<&OsStr>,
    file_name: &str,
) -> DriverLocation {
    if let Some(explicit) = explicit {
        return DriverLocation {
            path: PathBuf::from(explicit),
            origin: DriverOrigin::Override,
        };
    }

    if let Some(sibling) = front_end_dir
        .map(|dir| dir.join(file_name))
        .filter(|sibling| sibling.is_file())
    {
        return DriverLocation {
            path: sibling,
            origin: DriverOrigin::Sibling,
        };
    }

    let on_path = path_var.and_then(|path_var| {
        env::split_paths(path_var)
            .map(|dir| dir.join(file_name))
            .find(|candidate| candidate.is_file())
    });
    match on_path {
        Some(path) => DriverLocation {
            path,
            origin: DriverOrigin::Path,
        },
        None => DriverLocation {
            path: PathBuf::from(file_name),
            origin: DriverOrigin::Missing,
        },
    }
}
