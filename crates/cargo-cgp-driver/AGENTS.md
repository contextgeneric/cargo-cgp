# AGENTS.md — cargo-cgp-driver (rustc wrapper)

This crate is the `rustc_driver`-based compiler wrapper that `cargo-cgp` sets as the
`RUSTC_WORKSPACE_WRAPPER`. Read the workspace [../../AGENTS.md](../../AGENTS.md) first for the
two-binary architecture, the toolchain requirements, and the `rustc_private` gotchas; this file
covers only what is specific to the driver.

The driver runs the real compiler in-process. Cargo invokes it as
`cargo-cgp-driver <path-to-rustc> <rustc args...>`, and [`run::run`](src/run.rs) drives
`rustc_driver::run_compiler` under [`catch_with_exit_code`](src/run.rs), returning the compiler's
`ExitCode`. [`args::rustc_args`](src/args.rs) prepares the argument vector: it detects wrapper mode
(the second argument is a path whose stem is `rustc`), drops that injected path, and injects
`--sysroot` from `CARGO_CGP_SYSROOT` unless one is already present, because the driver lives outside
any toolchain and `rustc` cannot otherwise find `std`. The shared names live in
[`config.rs`](src/config.rs).

The real purpose of the crate lives in [`callbacks.rs`](src/callbacks.rs). `CgpCallbacks` is
currently an empty [`rustc_driver::Callbacks`] implementation, which is why the driver compiles
identically to plain `rustc` today. This is the extension point: overriding a callback such as
`config` or `after_analysis` to read the compiler's diagnostics and re-present CGP errors will hook
in here, without changing how the driver is wired into cargo. When you build that out, add the
`extern crate rustc_*;` lines a new compiler crate needs to [`lib.rs`](src/lib.rs), and consult the
[CGP error catalog](../../../cgp/docs/errors/README.md) for the error classes to recognize.

Two `rustc_private` constraints are non-negotiable. The `#![feature(rustc_private)]` gate must stay
on **both** [`lib.rs`](src/lib.rs) and the binary [`bin/cargo-cgp-driver.rs`](bin/cargo-cgp-driver.rs)
— the binary is what links the compiler dylib. And the driver embeds the pinned nightly's compiler,
so it must be run against a project using that same nightly; a sysroot from a different toolchain
loads a mismatched `librustc_driver` and fails.
