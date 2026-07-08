//! Well-known names shared with the front-end.

/// Environment variable through which `cargo-cgp` hands us the active toolchain's
/// sysroot. It is the counterpart of `cargo_cgp::config::SYSROOT_ENV`; the two crates
/// declare it independently and the shared string is the contract between them.
pub const SYSROOT_ENV: &str = "CARGO_CGP_SYSROOT";

/// The rustc flag that sets the sysroot. We inject it (with the value from
/// [`SYSROOT_ENV`]) only when cargo has not already passed one, because rustc cannot
/// infer a sysroot from the driver's out-of-tree location.
pub const SYSROOT_FLAG: &str = "--sysroot";

/// The flag that turns on the next-generation trait solver, injected into every
/// workspace-crate compilation.
///
/// This is how cargo-cgp surfaces the CGP dependency errors the default solver hides.
/// When a provider's impl-side dependency is unmet and the failure is reached by a
/// consumer-method call, the old solver's method-resolution heuristic bottoms out at the
/// provider trait (e.g. `Person: Greeter<Person>`) and never reports the real missing
/// leaf bound (e.g. `Person: HasField<Symbol!("name")>`). The next-gen solver descends to
/// that leaf — and renders CGP's own `#[diagnostic::on_unimplemented]` hint — so simply
/// compiling under it un-hides the root cause. We inject it unless the invocation already
/// sets `-Znext-solver`, so an explicit choice wins.
pub const NEXT_SOLVER_FLAG: &str = "-Znext-solver=globally";
