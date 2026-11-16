//! The compiler flags the driver injects into every workspace-crate compilation.

/// The flag that turns on the next-generation trait solver, injected into every workspace-crate
/// compilation unless the invocation already sets `-Znext-solver`.
///
/// See cgp-knowledge-base/cargo-cgp/implementation/driver/program.md for the cause it un-hides.
pub const NEXT_SOLVER_FLAG: &str = "-Znext-solver=globally";

/// The stable `--verbose` flag, injected into every workspace-crate compilation unless the
/// invocation already passes it, to stop the diagnostic machinery from *eliding* the parts of a
/// type it deems uninteresting.
///
/// See cgp-knowledge-base/cargo-cgp/implementation/driver/program.md for the three elisions it
/// turns off.
pub const VERBOSE_FLAG: &str = "--verbose";

/// The short form of [`VERBOSE_FLAG`].
pub const VERBOSE_SHORT_FLAG: &str = "-v";
