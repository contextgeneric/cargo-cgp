# cargo-cgp

`cargo-cgp` is a cargo subcommand that will make [Context-Generic Programming
(CGP)](https://github.com/contextgeneric/cgp) compiler errors readable. CGP macros expand to
ordinary Rust, so a small mistake in wiring can surface as a wall of errors that name generated
types the programmer never wrote, often with the real cause buried or suppressed. The goal of this
tool is to post-process those diagnostics into a compact, root-cause-first form — much as Clippy
layers its own analysis on top of `rustc`.

This repository is at an early stage. Today it ships one command, `cargo cgp check`, which forwards
to `cargo check` unchanged: the output is exactly what `cargo check` would print. That sounds like a
no-op, and behaviourally it is — but it establishes the mechanism the rest of the tool is built on.
`cargo cgp check` compiles your workspace through a custom `rustc` wrapper that has full access to
the compiler's internals via the `rustc_driver` API, which is the hook future versions will use to
read and rewrite CGP diagnostics.

## How it works

`cargo-cgp` follows the same two-binary design as Clippy. The `cargo-cgp` binary is the cargo
subcommand you invoke; it runs `cargo check` with the environment variable
`RUSTC_WORKSPACE_WRAPPER` pointed at the second binary, `cargo-cgp-driver`. Cargo then calls
`cargo-cgp-driver` in place of `rustc` for each crate in your workspace, while leaving dependencies
to compile with the normal compiler. The driver runs the real compiler in-process through
`rustc_driver`, so it sees everything `rustc` sees.

Because the driver links the compiler's internal libraries, it must be built with a nightly
toolchain that carries the `rustc-dev` component. That toolchain is pinned in
[`rust-toolchain.toml`](rust-toolchain.toml) and installs automatically the first time you build. It
is unrelated to the toolchain your own project uses — `cargo-cgp` only wraps whichever compiler
cargo already selects for the project being checked.

## Building and running

Build both binaries from the workspace root:

```sh
cargo build
```

To run the tool against another project during development, put the built binaries on your `PATH`
(they must sit in the same directory, since `cargo-cgp` locates the driver as its sibling) and run
the subcommand from that project:

```sh
cargo cgp check
```

Any arguments after `check` are forwarded verbatim to `cargo check`, so `cargo cgp check -v` or
`cargo cgp check --workspace` work as expected. The command can also be run directly as
`cargo-cgp check`.

## Status and roadmap

The current release is the scaffold: a working `cargo cgp check` that transparently wraps
`cargo check` with a `rustc_driver`-based driver, plus the project structure and documentation to
grow it. The next steps are to read the compiler's diagnostics inside the driver's callbacks,
recognize the CGP error classes catalogued in the upstream
[CGP error catalog](https://github.com/contextgeneric/cgp/tree/main/docs/errors), and re-present
them with the root cause first. Contributors and agents should start from
[AGENTS.md](AGENTS.md), which maps the code and records the conventions this project follows.

## License

Licensed under the MIT license.
