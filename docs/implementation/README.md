# cargo-cgp Implementation Reference

This directory documents the *internals* of `cargo-cgp` — how the tool is built, not how it is used.
It is the documentation an agent reviewing, debugging, or extending the source reads first: it
records the current state of the code in one place, so an agent can pick up a subsystem from where
the last one left off rather than reconstructing it from the two crates each time. The authoring
rules, the document shape, and the synchronization rule that binds these documents to the code live
in [AGENTS.md](AGENTS.md), on top of the knowledge-base-wide rules in [../AGENTS.md](../AGENTS.md).

Each document explains how one subsystem *works* and *why it is built that way*, and — because
`cargo-cgp` is modeled on Clippy and built on the compiler's unstable API — how the design compares
to Clippy and where it depends on `rustc_driver` behavior. The comparison is not decoration: Clippy
is the closest working example of the same integration, so knowing where the two agree and where
they diverge is often the fastest way to understand why a piece of code is shaped the way it is.

## Catalog

This section indexes every implementation document. When you add a document, register it here in the
same change.

- [Executable structure](executable-structure.md) — the two-executable split (the `cargo-cgp`
  front-end and the `cargo-cgp-driver` rustc wrapper), how the front-end wraps `cargo`, the
  environment contract between the two, and how the split compares to Clippy.
- [The driver](driver.md) — the deep dive into `cargo-cgp-driver`: how cargo invokes it, how it
  prepares the rustc argument vector, how it reaches the compiler through the `rustc_private`
  `rustc_driver` API, and the three transformations it applies to the diagnostics — the two flag
  levers (trait solver, verbosity) and the emitter that renames CGP wiring notes using the live
  compiler.
- [The error pipeline](error-pipeline.md) — the four-stage flow that turns rustc's raw diagnostics
  into readable CGP errors (configure rustc, capture, process, render), and the detail of the two
  compilation-side stages: the current flag injections that un-hide and un-elide CGP errors, and the
  planned capture stage that will collect diagnostics for processing.
- [Error processing](error-processing.md) — the stateless `process_cgp_errors` stage (in the
  `cargo-cgp-error-processing` crate) that transforms captured diagnostics into a smaller,
  root-cause-first set of CGP errors: its interface, its `cargo_metadata::Diagnostic` input and
  `CgpDiagnostic` output types, why it must be a stateful analysis rather than a per-error map, and how
  it is tested without running the tool. Its per-diagnostic preprocessing pipeline (stripping CGP path
  prefixes, resugaring `Symbol!`, rewriting unmet `HasField` bounds into missing-field messages) is
  built and wired in; the cross-diagnostic aggregation that collapses cascades is future work.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — a map of the compiler code that
  builds CGP diagnostics and, crucially, where it *suppresses* information: the type/const printer, the
  trait-error reporters, the two verbosity switches (`--verbose` versus `-Zverbose-internals`), and the
  specific elision points the driver's `--verbose` injection defeats. Read it when a diagnostic is
  dropping a cause the tool needs.
- [Testing](testing.md) — how the tool is tested: tests over the argument handling, and the UI
  snapshot suite — a custom Rust test harness (like Clippy's `compile-test`) that compiles fixtures
  under `tests/ui/` through the tool and diffs committed `.stderr` snapshots — its bless workflow,
  and how it compares to Clippy's harness.
