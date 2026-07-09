# AGENTS.md — cargo-cgp (front-end)

This crate is the `cargo-cgp` cargo-subcommand front-end — the binary a user invokes, which runs
`cargo check` with the driver wired in as the workspace rustc wrapper. Read the workspace
[../../AGENTS.md](../../AGENTS.md) for the project's goals, and
[Executable structure](../../docs/implementation/executable-structure.md) for how the front-end and
driver cooperate in full — the argument normalization, the environment contract, and the driver
lookup. This file covers only what is specific to this crate.

One rule is load-bearing: the front-end is a plain `std` + `anyhow` binary and **must not** depend on
`rustc_private` or on the driver crate as a library. It finds the driver executable on disk at
runtime, as a sibling of itself, the way `cargo-clippy` finds `clippy-driver`. Keeping compiler
internals out of this crate is what keeps it small; do not pull them in. Its diagnostic dependencies —
`cargo-cgp-error-processing` and `cargo_metadata` — are chosen to respect this rule: both are ordinary
crates that never touch `rustc_private`.

The module layout is short. [`run.rs`](src/run.rs) is the entrypoint — called by the thin
[`bin/cargo-cgp.rs`](bin/cargo-cgp.rs) — that normalizes arguments and dispatches on the subcommand;
[`args.rs`](src/args.rs) strips the inserted `cgp` token; [`config.rs`](src/config.rs) holds the
well-known names, passed into functions as parameters rather than hardcoded; and
[`check/`](src/check) implements the one subcommand, split into [`command.rs`](src/check/command.rs)
(runs the wrapped `cargo check` with `--message-format=json`, captures its output, and re-emits the
processed diagnostics), [`diagnostics.rs`](src/check/diagnostics.rs) (parses cargo's JSON stream into
`cargo_metadata::Diagnostic`s and re-renders the processed result),
[`driver_path.rs`](src/check/driver_path.rs) (locates the sibling driver), and
[`sysroot.rs`](src/check/sysroot.rs) (discovers the toolchain sysroot). The diagnostic transform
itself lives in the separate [`cargo-cgp-error-processing`](../cargo-cgp-error-processing) crate, which
this crate calls; the two stages this crate owns are *capture* and *render*, documented in
[The error pipeline](../../docs/implementation/error-pipeline.md).

When you add a subcommand, add its handler as a sibling of `check`, dispatch to it from `run`, and
keep the `bin` wrapper untouched. Cover argument handling with tests in the crate's `tests/`
directory (never inline in `src/`, per [../../AGENTS.md](../../AGENTS.md)), as
[`tests/args.rs`](tests/args.rs) does.
