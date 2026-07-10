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
injection, and injecting the flags in [`config.rs`](src/config.rs)); `config.rs` holds the shared
names and the injected flags; [`callbacks.rs`](src/callbacks.rs) holds the `Callbacks` implementation,
whose `config` hook installs the rewriting emitter; and [`emitter.rs`](src/emitter.rs),
[`component_map.rs`](src/component_map.rs), and [`rewrite.rs`](src/rewrite.rs) make up the
diagnostic-renaming transform.

The driver affects diagnostics in two ways. It injects `-Znext-solver=globally`
([`config::NEXT_SOLVER_FLAG`](src/config.rs)) and `--verbose` to configure how the compiler produces
diagnostics — a coarse, parse-free lever. And, through `callbacks.rs`, it installs a custom diagnostic
emitter that *rewrites* diagnostics the compiler has already built: [`emitter.rs`](src/emitter.rs)
reaches the live `TyCtxt` (from thread-local scope, valid because a wiring note is built during trait
solving), [`component_map.rs`](src/component_map.rs) inverts the `IsProviderFor` supertrait and
consumer-blanket-impl links into a component-marker → trait-names map, and the compiler-free
[`rewrite.rs`](src/rewrite.rs) renames the wiring notes. This is the enrichment front-end capture
cannot do, because it needs facts only the live compiler holds; the front-end still handles the
text-only rewrites over cargo's `--message-format=json` output. The transform is documented in
[The error pipeline](../../docs/implementation/error-pipeline.md#naming-the-traits-behind-a-component-marker-current);
the stateless front-end stage is [Error processing](../../docs/implementation/error-processing.md).
The `after_analysis` callback and an `InferCtxt`-reconstructed obligation chain remain future levers
on the same seam. When a new module needs a further compiler crate, add its `extern crate rustc_*;`
line to [`lib.rs`](src/lib.rs), and consult the
[CGP error catalog](../../../cgp/docs/errors/README.md) for the error classes to recognize.

Two `rustc_private` constraints are non-negotiable and easy to break: the
`#![feature(rustc_private)]` gate must stay on **both** [`lib.rs`](src/lib.rs) and the binary
[`bin/cargo-cgp-driver.rs`](bin/cargo-cgp-driver.rs) — the binary links the compiler dylib — and the
driver embeds the pinned nightly's compiler, so it must run against a project on that same nightly.
[Executable structure](../../docs/implementation/executable-structure.md#accessing-the-rust-compiler-api)
explains both.
