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
  `rustc_driver` API, and the transformations it applies to the diagnostics — the two flag levers
  (trait solver, verbosity), the generic `CgpEmitter` that renders text or JSON like vanilla `rustc`,
  the emitter that renames CGP wiring notes using the live compiler, and the post-processing fallback
  that keeps un-rewritten CGP constructs readable.
- [Typed root-cause resolution](typed-root-cause-resolution.md) — the deeper emitter transformation
  that turns a check-failure diagnostic into its root-cause `cargo tree`: recovering the chain by
  re-running the check obligation through the trait solver (from inside `emit_diagnostic`, since
  `after_analysis` is unreachable once the crate has errors), descending the wiring to each terminal
  leaf (a `HasField` field or an ordinary bound), and rendering the transitive dependency chain with
  each wiring trait replaced by its human form. A main message identified as a CGP class is rewritten
  and stamped with its `[CGP-Exxx]` code (the Rust code kept); the sub-notes become one `root cause:`
  note per leaf over its chain; anything it declines falls back to the text rewrite.
- [The error pipeline](error-pipeline.md) — the flow that turns rustc's raw diagnostics into readable
  CGP errors, now entirely inside the driver (configure rustc, transform each diagnostic, render text
  or JSON), with the front-end forwarding cargo's output untouched.
- [Error processing](error-processing.md) — the rustc-free `cargo-cgp-error-processing` crate that
  holds the driver's string-level diagnostic logic: the post-processing text transforms (stripping
  CGP path prefixes, resugaring `Symbol!` and `Path!`, rewriting unmet `HasField` bounds into
  missing-field messages), the wiring-message rewrite, the root-cause diagnosis model and the wording
  that turns it into the header, help, and note text, and the dependency-tree renderer, all driven by
  the driver's emitter and unit-tested without a compiler.
- [rustc diagnostic internals](rustc-diagnostic-internals.md) — a map of the compiler code that
  builds CGP diagnostics and, crucially, where it *suppresses* information: the type/const printer, the
  trait-error reporters, the two verbosity switches (`--verbose` versus `-Zverbose-internals`), and the
  specific elision points the driver's `--verbose` injection defeats. Read it when a diagnostic is
  dropping a cause the tool needs.
- [Testing](testing.md) — how the tool is tested: tests over the argument handling, and the UI
  snapshot suite — a custom Rust test harness (like Clippy's `compile-test`) that compiles fixtures
  under `tests/ui/` through the tool and diffs committed `.cgp.stderr` snapshots — its bless workflow,
  and how it compares to Clippy's harness.
- [Distribution](distribution.md) — the blueprint (ahead of implementation) for packaging and
  installing the tool on a machine that has only stable Rust: installing both binaries through cargo
  in lockstep, provisioning the pinned nightly with `rustc-dev`, forcing that nightly for the check so
  the project needs no toolchain of its own, and the version preflight that keeps front-end and driver
  matched. Follows the `rustc_plugin` model where Clippy's in-toolchain distribution is closed to an
  out-of-tree tool, and covers the `setup`/`update` subcommands (all provisioning confined to `setup`,
  a read-only preflight in `check`, cargo-delegated update with crates.io version discovery) and
  running the tool as a Rust Analyzer check backend (the JSON pipeline, the `RUSTC_WRAPPER`
  non-collision, and the isolated target directory).
