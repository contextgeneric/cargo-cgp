# cargo-cgp-driver

`cargo-cgp-driver` is the `rustc` driver of [`cargo-cgp`](https://crates.io/crates/cargo-cgp), the
cargo subcommand that makes Context-Generic Programming (CGP) compiler errors readable. The
`cargo-cgp` front-end runs it as cargo's workspace compiler wrapper; it is not a tool to run on its
own.

Install it through the front-end rather than directly:

```sh
cargo install --locked cargo-cgp
cargo cgp setup
```

The driver links the compiler's internal crates, so it must be built under the exact nightly
toolchain `cargo-cgp` pins, and at the front-end's own version. `cargo cgp setup` installs that
toolchain and builds this crate under it.

See the [`cargo-cgp` repository](https://github.com/contextgeneric/cargo-cgp) for how the tool
works.
