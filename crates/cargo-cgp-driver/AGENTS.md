# AGENTS.md — cargo-cgp-driver (rustc wrapper)

This crate is the `rustc_driver`-based compiler wrapper that `cargo-cgp` sets as the
`RUSTC_WORKSPACE_WRAPPER`, running the real compiler in-process. Read the workspace
[../../AGENTS.md](../../AGENTS.md) for the project's goals, and
[Executable structure](../../docs/implementation/executable-structure.md) for how the driver wraps
`rustc`, reaches the compiler API, and compares to Clippy. This file covers only what is specific to
this crate.

The module layout is short. [`run.rs`](src/run.rs) is the entrypoint — called by the thin
[`bin/cargo-cgp-driver.rs`](bin/cargo-cgp-driver.rs) — that drives `rustc_driver::run_compiler`;
[`args.rs`](src/args.rs) prepares the rustc argument vector (wrapper-mode stripping, sysroot
injection); [`config.rs`](src/config.rs) holds the shared names; and
[`callbacks.rs`](src/callbacks.rs) holds the `Callbacks` implementation.

`callbacks.rs` is where the crate's real purpose will live. `CgpCallbacks` is an empty
`rustc_driver::Callbacks` today, which is why the driver compiles identically to plain `rustc`. It is
the extension point: overriding a callback such as `config` or `after_analysis` to read diagnostics
and re-present CGP errors hooks in here. When you build that out, add the `extern crate rustc_*;`
lines a new compiler crate needs to [`lib.rs`](src/lib.rs), and consult the
[CGP error catalog](../../../cgp/docs/errors/README.md) for the error classes to recognize.

Two `rustc_private` constraints are non-negotiable and easy to break: the
`#![feature(rustc_private)]` gate must stay on **both** [`lib.rs`](src/lib.rs) and the binary
[`bin/cargo-cgp-driver.rs`](bin/cargo-cgp-driver.rs) — the binary links the compiler dylib — and the
driver embeds the pinned nightly's compiler, so it must run against a project on that same nightly.
[Executable structure](../../docs/implementation/executable-structure.md#accessing-the-rust-compiler-api)
explains both.
