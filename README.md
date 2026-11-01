# cargo-cgp

`cargo-cgp` is a cargo subcommand that makes the compiler errors of
[Context-Generic Programming (CGP)](https://github.com/contextgeneric/cgp) readable. `cargo cgp check`
stands in for `cargo check`, compiling your workspace through a `rustc` wrapper that recognizes CGP
errors and presents them root cause first, much as Clippy layers its own analysis on top of `rustc`.
`cargo cgp expand` shows the Rust your CGP macros generate, with CGP's type-level constructs spelled
the way you wrote them.

CGP itself is an ordinary library on the stable toolchain. It lets one trait have many interchangeable
implementations and lets each type choose which one it uses: you write that choice as *wiring* in one
place, and the compiler resolves it statically, at no runtime cost.

## What it solves

A CGP macro expands to ordinary Rust, so the compiler type-checks the *generated* code, not the code
you wrote. When a provider needs a value its context does not supply, the mistake surfaces as an error
about types you never typed, under machinery names like `IsProviderFor` and `CanUseComponent`, with
the missing field written as a nested `Symbol<6, Chars<…>>` spine or absent from the output. The tool
reads those diagnostics inside the compiler and rewrites them to name the real cause and the
dependency chain that leads to it.

A small program with one deliberate mistake shows the difference. A provider computes a rectangle's
area from `width` and `height` fields, but the `Rectangle` context has no `height`:

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

Plain `cargo check` reports an unsatisfied bound on a generated check trait, and leaves you to decode
the field from the spine and the chain from its notes:

```text
error[E0277]: the trait bound `Rectangle: CanUseComponent<AreaCalculatorComponent>` is not satisfied
   |
   |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `HasField<Symbol<6, Chars<'h', Chars<'e', Chars<'i', Chars<'g', Chars<_, cgp::prelude::Chars<'t', Nil>>>>>>>>` is not implemented for `Rectangle`
      but trait `HasField<Symbol<5, Chars<'w', Chars<'i', Chars<'d', Chars<'t', Chars<_, Nil>>>>>>>` is implemented for it
note: required for `RectangleArea` to implement `IsProviderFor<AreaCalculatorComponent, Rectangle>`
   = note: required for `Rectangle` to implement `CanUseComponent<AreaCalculatorComponent>`
note: required by a bound in `__CheckRectangle`
```

`cargo cgp check` names the missing field in its headline, shows the chain that requires it, and says
what to add:

```text
error[E0277]: [CGP-E001] missing field `height` on `Rectangle`
   |
   |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E110] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E110] missing field `height` on `Rectangle`
   = help: add the field `height: f64` to `Rectangle`
```

The Rust error code stays, so `rustc --explain E0277` still works, and the
[error-code catalog](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/error-code.md) explains each `[CGP-Exxx]` code. An error the tool does not
recognize passes through as `rustc` wrote it.

The check turns on the next-generation trait solver, which surfaces CGP dependency errors the default
solver hides, so its errors can differ from those of plain `cargo check`. The tool adds no way to
build, run, or test, so keep using plain cargo for those.

## Installing

The tool is two binaries, the `cargo-cgp` front-end you invoke and a `cargo-cgp-driver` that links the
compiler's internals, plus the exact nightly toolchain the driver is built against. It forces that
nightly only for its own check, so your project keeps its toolchain. Install with cargo if you have
rustup, or with Nix.

### With cargo

The front-end builds under the toolchain you already have, which must be Rust 1.96 or newer. Then
`cargo cgp setup` installs the pinned nightly and builds the matching driver against it:

```sh
cargo install --locked cargo-cgp
cargo cgp setup
```

### With Nix

The flake at the repository root builds both binaries against the pinned nightly and wraps them so
they run without rustup:

```sh
nix profile install github:contextgeneric/cargo-cgp/v0.1.0
```

To try the tool without installing it, run the flake from your project's directory. Everything after
`--` goes to the check, and your project's toolchain and `target/` stay untouched:

```sh
nix run github:contextgeneric/cargo-cgp/v0.1.0 -- check
```

The [installation guide](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/reference/installation.md) covers updating, uninstalling, and installing
from a source checkout.

## Expanding the generated code

`cargo cgp expand` prints one target after macro expansion, with type-level constructs restored to
macro notation: a field tag reads `Symbol!("width")` rather than the raw `Symbol<5, Chars<…>>` spine.
Reach for it when an error names types you never wrote and you need to see the impl behind them.

```sh
cargo cgp expand --lib                              # the whole library target
cargo cgp expand --lib --item contexts::MockApp     # one module, type, or trait
cargo cgp expand --help                             # the options
```

A package with a library and a binary needs `--lib` or `--bin NAME`, and every other argument goes to
`cargo rustc`. `--item <path>` narrows the output: a module gives its contents, a type its declaration
and every impl written for it, and a trait its definition and every impl of it. That last form suits a
component, whose generated items are almost all impls. The command is not a check, since it stops once
the macros are expanded, so its success says nothing about your wiring.

## Editor integration

Rust Analyzer can run `cargo cgp check` as its on-save check, so the rewritten errors appear inline.
The command is two words and must emit JSON, so wire it through `check.overrideCommand`:

```jsonc
"rust-analyzer.check.overrideCommand": [
  "cargo", "cgp", "check", "--workspace", "--all-targets", "--message-format=json"
]
```

The check builds into its own `target/cgp` directory, so it does not wait on your normal builds. The
[usage guide](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/reference/usage.md#editor-integration-rust-analyzer) lists what this override
changes in the editor.

## Learn more

The [CGP knowledge base](https://github.com/contextgeneric/cgp-knowledge-base) holds this tool's
documentation, under its [`cargo-cgp/` section](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/README.md):

- [Usage](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/reference/usage.md): running the check, reading its output, and expanding a target.
- [Installation](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/reference/installation.md): every install, update, and uninstall path.
- [Error codes](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/error-code.md): the `[CGP-Exxx]` codes in the output.
- [Troubleshooting](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/reference/troubleshooting.md): what to do when the tool itself will not run.
- [Known issues](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/README.md): the errors the tool does not yet present well.
- [Implementation](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/README.md): the two binaries, the driver, and how diagnostics
  are transformed.

Contributors and agents start from
[AGENTS.md](https://github.com/contextgeneric/cargo-cgp/blob/main/AGENTS.md), and
[sibling-projects.md](https://github.com/contextgeneric/cargo-cgp/blob/main/sibling-projects.md) lists
the related repositories.

## License

Licensed under the MIT license.
