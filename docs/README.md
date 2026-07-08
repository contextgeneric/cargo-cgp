# cargo-cgp Knowledge Base

This directory is a knowledge base about `cargo-cgp`, written by and for AI coding agents. Its
purpose is to record how the tool works — how it is structured, why it is built the way it is, and
how it integrates with cargo and the Rust compiler — so that an agent can pick up the work from
where the last one left off without re-deriving that understanding from the source each time. The
[AGENTS.md](../AGENTS.md) at the repository root orients an agent in the code; this knowledge base
is the durable, version-controlled record that goes deeper and stays in sync with it.

## Why this exists

`cargo-cgp` integrates with two moving targets — cargo's subcommand and wrapper protocol, and the
compiler's unstable `rustc_driver` API — and neither is self-documenting from the source alone.
Reading [`crates/cargo-cgp`](../crates/cargo-cgp) and [`crates/cargo-cgp-driver`](../crates/cargo-cgp-driver)
tells you *what* each function does, but not why the two-executable split exists, why the front-end
must compute a sysroot the driver could seemingly find itself, or how the design compares to the
tool it is modeled on. That reasoning has to be reconstructed by whoever reads the code next. This
knowledge base captures the reconstruction once, in prose, so the next agent reads the conclusion
instead of rebuilding it.

The knowledge base is also a contract. When an agent changes how the tool is structured — the
argument handling, the environment variables the two executables agree on, the way the driver
accesses the compiler — the matching document is where the intended new behavior is stated in plain
language, so a reviewer can compare the prose against the code. Documentation that drifts out of
sync with the code is worse than none, so keeping it accurate is a hard requirement of any change;
the maintenance rules, including the synchronization rule, live in [AGENTS.md](AGENTS.md).

## How it is organized

The knowledge base is divided into top-level categories, and it will grow to hold more as the tool
does. Each category answers a different question, so a reader picks the one that matches their need
rather than reading in sequence.

There are two categories. The [implementation/](implementation/README.md) directory documents
the *internals* of the tool — how each executable is built, how they cooperate, and how the driver
reaches the compiler — for an agent reviewing, debugging, or extending the source. Its
[catalog](implementation/README.md#catalog) indexes every implementation document and tracks which
parts of the tool are covered.

The [issues/](issues/README.md) directory is the second category, and it tracks *work* rather than
describing the tool: the problems `cargo-cgp` is meant to solve but does not yet, foremost the CGP
error classes it does not yet handle. Every issue is backed by a fixture under
[`tests/ui/`](../tests/ui) that reproduces it — a class with no reproducing fixture counts as
resolved — and the issues split along one axis, whether the root cause is recoverable from the tool's
output at all. [Hidden root cause](issues/hidden-root-cause.md) is the tool-oriented sufficiency
question: the cases where no downstream consumer could identify the root cause from the output alone.
[Usability issues](issues/usability.md) is the human-oriented readability question: output that
carries the cause but buries it, foremost overly verbose messages. Unlike the other categories, its
entries describe absent behavior and are deleted as the tool closes each gap.

As the tool grows a user-facing surface and more moving parts, expect further categories to appear
alongside `implementation/` — a user guide to running the tool, and a reference for the CGP error
classes it learns to recognize (drawing on the upstream
[CGP error catalog](../../cgp/docs/errors/README.md)). Add a category by creating its directory with
a `README.md` and registering it here in the same change.
