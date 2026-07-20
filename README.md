# cargo-cgp

[Context-Generic Programming (CGP)](https://github.com/contextgeneric/cgp) is a language extension
for Rust, with pluggable trait implementations at compile-time. It is an ordinary library on the
stable toolchain that lets one trait have many interchangeable implementations and lets each type
choose which one it uses — you write that choice as *wiring* in one place, and the compiler resolves
it statically, so there is no runtime cost.

`cargo-cgp` is a cargo subcommand that makes CGP's compiler errors readable. It stands in for
`cargo check`, compiling your workspace through a `rustc` wrapper that recognizes CGP wiring errors
and re-presents them with the root cause first — much as Clippy layers its own analysis on top of
`rustc`.

> **This is a pre-release (`v0.1.0-alpha`).** The tool works and is useful today, but its surface is
> small and still changing. See [Status](#status) for what it does now.

## What cargo-cgp solves

A CGP macro expands to ordinary Rust, so the compiler type-checks the *generated* code, not the code
you wrote. When a provider needs a value its context does not supply, the mistake surfaces as an
error about types you never typed — buried under machinery names like `IsProviderFor` and
`CanUseComponent`, with the actual missing field written as a nested `Symbol<6, Chars<..>>` spine, or
hidden from the output entirely. `cargo-cgp` reads those diagnostics inside the compiler and rewrites
them into a compact form that names the real cause and the dependency chain that leads to it.

The difference is easiest to see on a small program with one deliberate mistake. Here a provider
computes a rectangle's area from `width` and `height` fields, but the `Rectangle` context is missing
its `height`:

```rust
use cgp::prelude::*;

#[cgp_component(AreaCalculator)]
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}

#[cgp_impl(new RectangleArea)]
impl AreaCalculator {
    fn area(&self, #[implicit] width: f64, #[implicit] height: f64) -> f64 {
        width * height
    }
}

#[derive(HasField)]
pub struct Rectangle {
    pub width: f64,
    // the `height` field the provider needs is missing:
    // pub height: f64,
}

delegate_components! {
    Rectangle {
        AreaCalculatorComponent: RectangleArea,
    }
}

check_components! {
    Rectangle {
        AreaCalculatorComponent,
    }
}
```

Plain `cargo check` reports this roughly as an unsatisfied bound on a generated check trait, and
leaves you to decode which field is missing from a `Symbol<..>` spine and a chain of `IsProviderFor`
notes:

```text
error[E0277]: the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied
  |
  |         AreaCalculatorComponent,
  |         ^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
  |
help: the trait `HasField<Symbol<6, Chars<..>>>` is not implemented for `Rectangle`
      but trait `HasField<Symbol<5, Chars<..>>>` is implemented for it
note: required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`
note: required by a bound in `__CheckRectangle`
   ...
```

`cargo cgp check` rewrites the same failure to name the missing field outright and show the
dependency chain that requires it, tagged with `[CGP-Exxx]` codes you can look up:

```text
error[E0277]: [CGP-E001] the consumer trait `CanCalculateArea` is not implemented for context `Rectangle`
  |
  |         AreaCalculatorComponent,
  |         ^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: root cause: [CGP-E106] missing field `height` on `Rectangle`
          this is required through the dependency chain:
            [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
            └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
              └─ [CGP-E106] missing field `height` on `Rectangle`
```

The Rust error code (`E0277`) is always kept, so `rustc --explain` still works; the `[CGP-Exxx]`
codes ride inside the message and are catalogued in
[docs/error-code.md](docs/error-code.md).

## Installing

`cargo-cgp` is two binaries — the `cargo-cgp` front-end you invoke and a `cargo-cgp-driver` that
links the compiler internals — plus the exact nightly toolchain the driver is built against. There
are two ways to install the set, depending on how your machine manages Rust: with cargo if you have
rustup, or with Nix. cargo-cgp requires that exact nightly, which `cargo cgp setup` installs (or the
Nix flake builds against), and forces it only for its own check, so your own project keeps whatever
toolchain it already uses.

### With Nix

The flake at the repository root builds both binaries against the pinned nightly and wraps them so
they run without rustup. To install the tool onto your `PATH` so `cargo cgp check` works like any
other cargo subcommand, install the pre-release tag from the profile:

```sh
nix profile install github:contextgeneric/cargo-cgp/v0.1.0-alpha
```

### With cargo

The cargo path installs the small front-end first, then lets it provision the pinned nightly and the
matching driver in a second step:

```sh
cargo install cargo-cgp      # installs the front-end (builds on any toolchain)
cargo cgp setup              # installs the pinned nightly + driver, in lockstep
```

The first command builds the front-end under whatever toolchain you already have; `cargo cgp setup`
then provisions the pinned nightly and builds the matching driver against it, in lockstep. Later,
`cargo cgp update` upgrades the tool. Full instructions for every path, including installing from a
source checkout, are in
[docs/reference/installation.md](docs/reference/installation.md).

## Running it without installing

To try the tool on a project without adding anything to your `PATH`, run the flake's default app from
that project's directory, pinning the pre-release tag:

```sh
cd /path/to/your/project     # a cargo package or workspace that uses `cgp`
nix run github:contextgeneric/cargo-cgp/v0.1.0-alpha -- check
```

This builds (or reuses a cached) `cargo-cgp` and `cargo-cgp-driver` under the pinned nightly and runs
`cargo cgp check` in the current directory, with everything after `--` forwarded to the check. It
needs no rustup and leaves the project's own toolchain and `target/` untouched, which makes it
convenient for CI or a one-off trial. To pin the tool as an input in another project's own flake, add
`inputs.cargo-cgp.url = "github:contextgeneric/cargo-cgp/v0.1.0-alpha";` and take its
`packages.default`.

## Uninstalling

How you uninstall matches how you installed.

If you installed with **Nix**, remove it from your profile:

```sh
nix profile remove cargo-cgp
```

If you installed with **cargo**, uninstall both binaries:

```sh
cargo uninstall cargo-cgp cargo-cgp-driver
```

The pinned nightly toolchain that `cargo cgp setup` installed is left in place, since other tools may
depend on it. Remove it by hand if nothing else needs it (its name is printed by
`cargo-cgp-driver --version`):

```sh
rustup toolchain uninstall <pinned-nightly>
```

## Editor integration (Rust Analyzer)

Rust Analyzer can run `cargo cgp check` as its on-save check backend, so the rewritten CGP errors
appear inline in your editor. Because the command is two words and must emit JSON, wire it through
`check.overrideCommand` (not `check.command`) with `--message-format=json`:

```jsonc
"rust-analyzer.check.overrideCommand": [
  "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
]
```

The check builds into an isolated `target/cgp` directory, so it will not contend with your normal
builds or Rust Analyzer's own project loading. The full integration notes are in
[docs/reference/usage.md](docs/reference/usage.md#editor-integration-rust-analyzer).

## Status

This is an early pre-release. Today the tool ships one command, `cargo cgp check`, which stands in for
`cargo check` and does two things for CGP code: it turns on the **next-generation trait solver**,
which surfaces the CGP dependency errors the default solver hides, and it **rewrites the wiring errors
it recognizes** into the root-cause-first form shown above. Errors it does not yet recognize pass
through unchanged. The set of recognized error classes will grow over the pre-release series.

## Learn more

The `docs/` directory is a knowledge base covering both how to use the tool and how it is built:

- [docs/reference/usage.md](docs/reference/usage.md) — running the check, reading its output, and
  editor integration.
- [docs/reference/installation.md](docs/reference/installation.md) — every install, update, and
  uninstall path.
- [docs/error-code.md](docs/error-code.md) — the catalog of the `[CGP-Exxx]` codes in the output.
- [docs/reference/troubleshooting.md](docs/reference/troubleshooting.md) — what to do when the tool
  itself will not run.
- [docs/README.md](docs/README.md) — the rest of the knowledge base, including the implementation
  internals (the two-binary design, the driver, and how the diagnostics are transformed).

Contributors and agents should start from [AGENTS.md](AGENTS.md), which maps the code and records the
conventions this project follows.

## License

Licensed under the MIT license.
