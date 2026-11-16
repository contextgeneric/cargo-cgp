//! The compiler flags the driver injects into every workspace-crate compilation.

/// The flag that turns on the next-generation trait solver, injected into every
/// workspace-crate compilation.
///
/// This is how cargo-cgp surfaces the CGP dependency errors the default solver hides.
/// When a provider's impl-side dependency is unmet and the failure is reached by a
/// consumer-method call, the default solver's method-resolution heuristic bottoms out at the
/// provider trait (e.g. `Person: Greeter<Person>`) and never reports the real missing
/// leaf bound (e.g. `Person: HasField<Symbol!("name")>`). The next-gen solver descends to
/// that leaf, and renders CGP's own `#[diagnostic::on_unimplemented]` hint, so simply
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
/// - When a type's printed form grows long (routine for CGP's deeply nested `Symbol` /
///   `Cons` spines), it truncates the type and writes the full form to a `long-type-*.txt`
///   file instead of the diagnostic.
/// - When more than nine impls could apply, it prints a few and collapses the rest to
///   "and N others".
///
/// `--verbose` turns all three off, so the full type is always present in the output.
/// Unlike `-Zverbose-internals` (which also flips `opts.verbose`), it does *not* enable
/// the compiler's internal debug printing (no disambiguator suffixes, no region ids), so
/// the diagnostics stay in their ordinary shape, only without the elisions. We inject it
/// unless the invocation already passes `--verbose`, so an explicit choice wins.
pub const VERBOSE_FLAG: &str = "--verbose";

/// The short form of [`VERBOSE_FLAG`].
pub const VERBOSE_SHORT_FLAG: &str = "-v";

/// The flag that runs rustc's front end on one thread, injected into every workspace-crate
/// compilation in place of any thread count the invocation carries.
///
/// From `nightly-2026-11-14`, rustc type-checks on several threads unless told otherwise, so
/// diagnostics from different items arrive in a different order on each run. The emitter processes
/// and prints them in a canonical order, so its output is meant to be the same either way, but no
/// parallel run has yet shown it; and a diagnostic it transforms outside `analysis` runs under
/// rustc's diagnostic lock, which under threads blocks on re-entry rather than panicking. One
/// thread keeps the output identical between runs until then. Unlike the other injected flags, a
/// user's own thread count does not win, since a `-Z threads=8` set in a cargo configuration for
/// every build would bring those hazards to the check: [`THREAD_COUNT_OPTIONS`] lists the spellings
/// removed first.
pub const THREADS_FLAG: &str = "-Zthreads=1";

/// The options that set the front end's thread count, each removed from the invocation before
/// [`THREADS_FLAG`] is appended. rustc rejects `--jobs-frontend` beside `-Zthreads`, so keeping a
/// user's `--jobs-frontend` would end the compilation.
pub const THREAD_COUNT_OPTIONS: &[&str] = &["-Zthreads", "--jobs-frontend"];
