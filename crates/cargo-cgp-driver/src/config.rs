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

/// The stable `--verbose` flag, injected into every workspace-crate compilation to stop
/// the diagnostic machinery from *eliding* the parts of a type it deems uninteresting.
///
/// rustc's error reporting compresses types in several ways that each drop information a
/// downstream consumer of the diagnostic needs, and all of them are gated on the single
/// `opts.verbose` flag that `--verbose` sets:
///
/// - When it reports "trait `X` is not implemented … but trait `Y` is" (the *similar
///   impl* hint), it diffs the two traits and replaces every generic argument the two
///   share with `_`. For CGP that erasure lands inside a `Symbol!` type: two field-name
///   symbols that share a character have that shared `char` printed as `_` in *both*
///   symbols, so the field name cannot be read back from the text at all.
/// - When a type's printed form grows long — routine for CGP's deeply nested `Symbol` /
///   `Cons` spines — it truncates the type and writes the full form to a `long-type-*.txt`
///   file instead of the diagnostic.
/// - When more than nine impls could apply, it prints a few and collapses the rest to
///   "and N others".
///
/// `--verbose` turns all three off, so the full type is always present in the output.
/// Unlike `-Zverbose-internals` (which also flips `opts.verbose`), it does *not* enable
/// the compiler's internal debug printing — no disambiguator suffixes, no region ids — so
/// the diagnostics stay in their ordinary shape, only without the elisions. We inject it
/// unless the invocation already passes `--verbose`, so an explicit choice wins.
pub const VERBOSE_FLAG: &str = "--verbose";
