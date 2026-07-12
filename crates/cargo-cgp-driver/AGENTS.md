# AGENTS.md — cargo-cgp-driver (rustc wrapper)

This crate is the `rustc_driver`-based compiler wrapper that `cargo-cgp` sets as the
`RUSTC_WORKSPACE_WRAPPER`, running the real compiler in-process. Read the workspace
[../../AGENTS.md](../../AGENTS.md) for the project's goals,
[Executable structure](../../docs/implementation/executable-structure.md) for how the two executables
cooperate, and [The driver](../../docs/implementation/driver.md) for the full deep dive into this
crate — how cargo invokes it, how it reaches the compiler API, and the three diagnostic
transformations. This file covers only the orientation specific to working in the crate.

The module layout is short. [`run.rs`](src/run.rs) is the entrypoint — called by the thin
[`bin/cargo-cgp-driver.rs`](bin/cargo-cgp-driver.rs) — that drives `rustc_driver::run_compiler`;
[`args.rs`](src/args.rs) prepares the rustc argument vector (wrapper-mode stripping, sysroot
injection, and injecting the flags in [`config.rs`](src/config.rs)); `config.rs` holds the shared
names and the injected flags; [`callbacks.rs`](src/callbacks.rs) holds the `Callbacks` implementation,
whose `config` hook installs the transforming emitter; [`emitter.rs`](src/emitter.rs),
[`resolve.rs`](src/resolve.rs), and [`component_map.rs`](src/component_map.rs) make up the
compiler-coupled half of the diagnostic transforms. The compiler-free half — the wiring rewrite, the
post-processing text transforms, and the `ComponentNameMap` — lives in the
`cargo-cgp-error-processing` crate (the driver's one ordinary dependency), so it can be unit-tested
without this crate's `rustc_private` linkage.

The driver does **all** of the tool's diagnostic work; the front-end merely forwards cargo's output.
It affects diagnostics in three ways. First it injects `-Znext-solver=globally`
([`config::NEXT_SOLVER_FLAG`](src/config.rs)) and `--verbose` to configure how the compiler produces
diagnostics — a coarse, parse-free lever. Second, through `callbacks.rs`, it installs a custom
emitter — [`emitter.rs`](src/emitter.rs)'s `CgpEmitter<E>`, generic over its inner emitter so it wraps
whichever the compiler's default would build (a `JsonEmitter` or an `AnnotateSnippetEmitter`) and
renders text or JSON like vanilla `rustc`. That emitter *rewrites* diagnostics the compiler has
already built: it reaches the live `TyCtxt` (from thread-local scope, valid because a wiring note is
built during trait solving), [`component_map.rs`](src/component_map.rs) inverts the `IsProviderFor`
supertrait (anchored by `DefId` identity to the `cgp_component` crate, not matched by name) and
consumer-blanket-impl links into a component-marker → trait-names map, wrapped in a lazily-built
`ComponentNameMap`, and [`resolve.rs`](src/resolve.rs) recovers a check failure's root-cause
dependency tree from the trait solver; the compiler-free `rewrite` module (in
`cargo-cgp-error-processing`) renames the wiring messages. Third, every diagnostic then goes through
the compiler-free `postprocess` transforms (strip CGP path prefixes, resugar `Symbol!`, reword an
unmet `HasField` bound), the final cleanup that keeps raw CGP constructs readable. The transforms are
documented in full in
[The driver](../../docs/implementation/driver.md#naming-the-traits-behind-a-component-marker) and
[Typed root-cause resolution](../../docs/implementation/typed-root-cause-resolution.md); the
compiler-free helpers are [Error processing](../../docs/implementation/error-processing.md). When a
new module needs a further compiler crate, add its `extern crate rustc_*;` line to
[`lib.rs`](src/lib.rs), and consult the
[CGP error catalog](../../../cgp/docs/errors/README.md) for the error classes to recognize.

Two `rustc_private` constraints are non-negotiable and easy to break: the
`#![feature(rustc_private)]` gate must stay on **both** [`lib.rs`](src/lib.rs) and the binary
[`bin/cargo-cgp-driver.rs`](bin/cargo-cgp-driver.rs) — the binary links the compiler dylib — and the
driver embeds the pinned nightly's compiler, so it must run against a project on that same nightly.
[The driver](../../docs/implementation/driver.md#accessing-the-rust-compiler-api)
explains both.
