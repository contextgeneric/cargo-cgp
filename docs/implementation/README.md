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
  front-end and the `cargo-cgp-driver` rustc wrapper), how the front-end wraps `cargo` and the
  driver wraps `rustc`, how the driver reaches the compiler through the `rustc_private`
  `rustc_driver` API, and how all of this compares to Clippy.
- [Testing](testing.md) — how the tool is tested: unit tests over the argument handling, the UI
  snapshot suite that compiles fixtures under `tests/ui/` through the tool and diffs committed
  `.stderr` snapshots, the `scripts/` harness and bless workflow, and how the setup compares to
  Clippy's UI-test harness.
