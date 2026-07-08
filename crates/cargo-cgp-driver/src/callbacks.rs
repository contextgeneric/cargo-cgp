//! The compiler callbacks the driver installs.

/// The driver's [`rustc_driver::Callbacks`] implementation.
///
/// It is currently empty, so compilation proceeds identically to plain `rustc`. This is
/// the extension point for cargo-cgp's real purpose: overriding callbacks such as
/// `config` (to adjust the compiler session) or `after_analysis` (to read diagnostics
/// and post-process CGP errors) will hook in here without changing how the driver is
/// wired into cargo.
pub struct CgpCallbacks;

impl rustc_driver::Callbacks for CgpCallbacks {}
