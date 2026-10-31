//! What a driver's `--version` reports.

/// The identity a `cargo-cgp-driver --version` reports, parsed from its output: the two fields the
/// preflight compares. The driver also prints its `pinned-toolchain:`, for a reader; the preflight
/// names the toolchain it queried instead, which [`crate::config::TOOLCHAIN_ENV`] can change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverVersion {
    /// The driver crate's version, which must equal the front-end's exactly.
    pub tool_version: String,
    /// The `rustc --version` line of the compiler that built the driver, which must equal the
    /// installed pinned toolchain's `rustc --version`.
    pub built_against_rustc: String,
}

/// Parse the driver's `--version` output into a [`DriverVersion`]. Returns `None` if the
/// shape is not what [`crate`]'s driver prints (an old or foreign binary).
pub fn parse_driver_version(output: &str) -> Option<DriverVersion> {
    let mut lines = output.lines();

    let tool_version = lines
        .next()?
        .strip_prefix("cargo-cgp-driver")?
        .trim()
        .to_owned();
    if tool_version.is_empty() {
        return None;
    }

    let built_against_rustc = lines
        .find_map(|line| line.strip_prefix("built-against-rustc:"))?
        .trim()
        .to_owned();

    Some(DriverVersion {
        tool_version,
        built_against_rustc,
    })
}
