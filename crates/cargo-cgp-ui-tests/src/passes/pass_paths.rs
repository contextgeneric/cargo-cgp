//! The paths a pass normalizes away.

use std::path::Path;

/// The machine-specific paths every pass normalizes away.
pub struct Paths<'a> {
    /// The worker's throwaway crate.
    pub harness_crate: &'a Path,
    /// The sibling `cgp` checkout.
    pub cgp_root: &'a Path,
    /// The toolchain sysroot.
    pub sysroot: &'a Path,
}
