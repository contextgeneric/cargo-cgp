[![Crates.io](https://img.shields.io/crates/v/cargo-cgp-driver.svg)](https://crates.io/crates/cargo-cgp-driver)
[![Docs.rs](https://docs.rs/cargo-cgp-driver/badge.svg)](https://docs.rs/cargo-cgp-driver)

# cargo-cgp-driver

**The compiler half of [`cargo-cgp`](https://crates.io/crates/cargo-cgp), CGP's error toolchain.**

Context-Generic Programming (CGP) extends Rust with pluggable trait implementations, chosen at
compile time. Its macros expand to ordinary Rust, so when a context lacks something a provider
needs, the compiler reports the mistake in terms of the generated code. `cargo cgp check` compiles
your workspace through this crate, a `rustc` driver that reads CGP's diagnostics inside the compiler
and rewrites them to lead with the cause. For a `Rectangle` context whose area provider reads a
`height` field it does not have, the driver reports the mistake at the context's `check_components!`
entry:

```text
error[E0277]: [CGP-E001] missing field `height` on `Rectangle`
  --> src/main.rs:54:9
   |
54 |         AreaCalculatorComponent,
   |         ^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: root cause: [CGP-E110] missing field `height` on `Rectangle`
           this is required through the dependency chain:
             [CGP-E101] consumer trait impl `CanCalculateArea` for context `Rectangle`
             └─ [CGP-E102] provider trait impl `AreaCalculator` with context `Rectangle` for provider `RectangleArea`
               └─ [CGP-E110] missing field `height` on `Rectangle`
   = help: add the field `height: f64` to `Rectangle`
```

`cargo cgp check` leads with the root cause for the classes it recognizes, and the tool does not yet
reshape every class.

The driver links the compiler's internal crates, so it builds only under the nightly toolchain
`cargo-cgp` pins, and must match the front end's own version. That is why you install it through the
front end rather than on its own. CGP itself compiles on stable Rust, and the pinned nightly serves
only the tool's check, so your project keeps its own toolchain:

```sh
cargo install --locked cargo-cgp
cargo cgp setup
```

The [`cargo-cgp` documentation](https://contextgeneric.dev/docs/tooling/cargo-cgp/) on the
[CGP website](https://contextgeneric.dev/) covers
[installation](https://contextgeneric.dev/docs/tooling/cargo-cgp/installation) and
[reading the output](https://contextgeneric.dev/docs/tooling/cargo-cgp/reading-output), and the rest
of the site teaches CGP itself.
