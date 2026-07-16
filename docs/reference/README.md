# cargo-cgp Reference

This directory is the reference an AI agent consults to install and use `cargo-cgp`: how to get the
tool running and how to drive its one command. Like the rest of this knowledge base, it is written by
and for coding agents — it is **not** end-user documentation; the separate guides written for human
readers live outside `docs/`. Its concern is *using* the tool, in contrast to the sibling
[implementation/](../implementation/README.md) category, which documents how the tool is built and
maintained for an agent changing its source. That usage-versus-internals split mirrors the one in the
parent project, whose [CGP construct reference](../../../cgp/docs/reference/README.md) documents how
to *use* each CGP construct while its implementation directory documents how each is *built* — here
the subject is the `cargo-cgp` command line rather than a set of macros. Each document is
self-contained, so read the one that matches your need rather than reading in order.

Because these documents are for an agent working in or beside a `cargo-cgp` checkout, they favor the
**local build** of the tool over a published release wherever it matters: a local checkout reflects
the current code, including uncommitted changes, which is what an agent testing or explaining the
tool needs. The install and usage documents both call out where to prefer the local version.

## Overview

Using `cargo-cgp` has two phases — getting it onto your machine, then running it — and the two
reference documents cover one each. Today the tool is a single command, `cargo cgp check`, which
compiles your workspace through a custom `rustc` wrapper so it can present CGP wiring errors with the
root cause first instead of the wall of generated-type errors the plain compiler prints.

Installation is shaped by the one fact that the tool embeds a compiler. The `cargo-cgp-driver` binary
links the compiler's internal libraries, so it must be built against an exact pinned nightly that
carries the `rustc-dev` component, and the `cargo-cgp` front-end must be able to find that driver
beside it. [installation.md](installation.md) covers the two ways to satisfy this: the cargo path,
where you install the front-end and then run `cargo cgp setup` to provision the pinned nightly and
build the matching driver through rustup; and the Nix path, where a flake builds both binaries
against the pinned nightly for you and needs no rustup at all. It also covers keeping the tool
current — `cargo cgp update` for the cargo path, a flake update for Nix.

Once the tool is installed, running it is deliberately close to running `cargo check`.
[usage.md](usage.md) covers `cargo cgp check`: how it forwards its arguments straight to
`cargo check`, why it forces the pinned nightly and builds into an isolated `target/cgp` directory so
it never disturbs the checked project's normal builds, and how to read the output — in particular the
`[CGP-Exxx]` codes it stamps on the errors it recognizes. It also covers wiring the tool in as a Rust
Analyzer check backend and running it against a project outside this repository — through Nix,
preferring a local `cargo-cgp` checkout over a published release — along with the environment
variables that override its default behavior.

When a run goes wrong, [troubleshooting.md](troubleshooting.md) is the diagnostic companion to the
other two. Because the tool is two binaries plus a pinned compiler in separate locations, a failure
can sit at any of several seams — the front-end not finding the driver, the driver not loading the
compiler library, a toolchain or version mismatch, or the check being run in the wrong place — and
the document pairs the exact error message each failure prints with its cause and fix, so an agent can
match a symptom and act rather than guess.

The tool is at an early stage, so the documents describe what exists now and note plainly where a
path is intended but not yet available. For the reasoning *behind* the behavior these documents
describe — why the toolchain is pinned, how the two binaries stay in lockstep, how the driver reaches
the compiler — follow the links into the [implementation/](../implementation/README.md) category,
which is the authoritative source those references summarize.

## Index

This section lists every reference document with a summary of what it explains. When you add a
document, register it here in the same change.

- [Installation](installation.md) — how to install and update `cargo-cgp`. Covers the cargo path
  (`cargo install cargo-cgp` followed by `cargo cgp setup` to provision the pinned nightly and the
  driver, then `cargo cgp update` to upgrade), the Nix path (running or installing the tool from the
  flake, which builds both binaries against the pinned nightly with no rustup needed), installing
  from source for development, and the current availability status of each path.
- [Usage](usage.md) — how to use `cargo-cgp` once installed. Covers the `cargo cgp check` command and
  its argument forwarding, what the check does differently from `cargo check` (the forced pinned
  nightly, the next-generation trait solver, the isolated `target/cgp` directory), how to read the
  transformed output and its `[CGP-Exxx]` codes, using the tool as a Rust Analyzer check backend,
  running it on a project outside this repository through Nix (preferring a local checkout), and the
  environment variables that override its behavior.
- [Troubleshooting](troubleshooting.md) — how to diagnose a `cargo-cgp` that will not run. Covers
  isolating the failing seam (the driver `--version` load test, `cargo cgp check -v`) and the common
  failures with the exact error each prints — command and cargo-project errors, the front-end failing
  to find the driver, the driver failing to load `librustc_driver` (unset library path or a toolchain
  mismatch), the managed preflight's toolchain/driver/version verdicts, and a missing rustup — plus a
  symptom-to-section index.
