# cargo-cgp

`cargo-cgp` is a cargo subcommand that will make [Context-Generic Programming
(CGP)](https://github.com/contextgeneric/cgp) compiler errors readable. CGP macros expand to
ordinary Rust, so a small mistake in wiring can surface as a wall of errors that name generated
types the programmer never wrote, often with the real cause buried or suppressed. The goal of this
tool is to post-process those diagnostics into a compact, root-cause-first form — much as Clippy
layers its own analysis on top of `rustc`.

This repository is at an early stage. Today it ships one command, `cargo cgp check`, which compiles
your workspace through a custom `rustc` wrapper built on the `rustc_driver` API. That wrapper already
does one useful thing: it turns on the **next-generation trait solver**, which surfaces CGP
dependency errors the default solver hides. When a provider needs, say, a `name` field the context
lacks, the default compiler reports only that a method's bounds went unsatisfied and never names the
missing field; under the next-gen solver the same mistake reports the real missing bound
(`HasField<Symbol!("name")>`) and even CGP's own "add `#[derive(HasField)]`" hint. Beyond that the
output still matches `cargo check` — but the `rustc_driver` foothold is the hook future versions will
use to read and rewrite CGP diagnostics further.

## How it works

`cargo-cgp` follows the same two-binary design as Clippy. The `cargo-cgp` binary is the cargo
subcommand you invoke; it runs `cargo check` with the environment variable
`RUSTC_WORKSPACE_WRAPPER` pointed at the second binary, `cargo-cgp-driver`. Cargo then calls
`cargo-cgp-driver` in place of `rustc` for each crate in your workspace, while leaving dependencies
to compile with the normal compiler. The driver runs the real compiler in-process through
`rustc_driver`, so it sees everything `rustc` sees — and it injects `-Znext-solver=globally` into
each workspace-crate compilation, which is what surfaces the otherwise-hidden dependency errors.

Because the driver links the compiler's internal libraries, it must be built with a nightly
toolchain that carries the `rustc-dev` component. That toolchain is pinned in
[`rust-toolchain.toml`](rust-toolchain.toml) and installs automatically the first time you build. It
is unrelated to the toolchain your own project uses — `cargo-cgp` only wraps whichever compiler
cargo already selects for the project being checked.

For the full internal picture — the argument handling, the environment contract between the two
executables, how the driver reaches the compiler API, a comparison with Clippy, and links to the
authoritative Cargo and rustc references — see
[docs/implementation/executable-structure.md](docs/implementation/executable-structure.md).

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

## Testing

The common commands, run from the workspace root:

```sh
cargo test                          # all tests: the tool crates' unit tests and the UI suite
cargo fmt --all -- --check          # formatting (uses nightly rustfmt settings)
cargo clippy --all-targets -- -D warnings   # lints

# The UI snapshot suite is part of `cargo test`. To work with it directly:
cargo test -p cargo-cgp-ui-tests                             # just the suite
cargo test -p cargo-cgp-ui-tests --test ui -- hidden         # filter by path substring
cargo test -p cargo-cgp-ui-tests --test ui -- --bless        # regenerate snapshots
```

The UI suite compiles example CGP programs through cargo-cgp and diffs the tool's output against
committed `.stderr` snapshots (see [docs/implementation/testing.md](docs/implementation/testing.md)).
Because it runs as part of `cargo test`, the test run builds the driver, expects a sibling `cgp`
checkout at `../cgp`, and uses the pinned nightly toolchain — so snapshots are reproducible only
under that toolchain, and a bump can require a re-bless.

## Status and roadmap

The current release wraps `cargo check` with a `rustc_driver`-based driver and uses it for a first
real win — enabling the next-gen trait solver to un-hide CGP dependency errors — plus the project
structure and documentation to grow it. The next steps are to read the compiler's diagnostics inside
the driver's callbacks, recognize the CGP error classes catalogued in the upstream
[CGP error catalog](https://github.com/contextgeneric/cgp/tree/main/docs/errors), and re-present
them with the root cause first. Contributors and agents should start from
[AGENTS.md](AGENTS.md), which maps the code and records the conventions this project follows.

## License

Licensed under the MIT license.
