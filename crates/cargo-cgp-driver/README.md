[![Crates.io](https://img.shields.io/crates/v/cargo-cgp-driver.svg)](https://crates.io/crates/cargo-cgp-driver)
[![Docs.rs](https://docs.rs/cargo-cgp-driver/badge.svg)](https://docs.rs/cargo-cgp-driver)

# cargo-cgp-driver: the rustc driver behind cargo-cgp

**The `rustc` driver of [`cargo-cgp`](https://crates.io/crates/cargo-cgp), the cargo subcommand that
makes Context-Generic Programming (CGP) compiler errors readable.**

The `cargo-cgp` front-end runs this crate as cargo's workspace compiler wrapper, so it is not a tool
to run on its own. Install it through the front-end rather than directly:

```sh
cargo install --locked cargo-cgp
cargo cgp setup
```

The driver links the compiler's internal crates, so it must be built under the exact nightly
toolchain `cargo-cgp` pins, and at the front-end's own version. `cargo cgp setup` installs that
toolchain and builds this crate under it.

**[Website](https://contextgeneric.dev/docs/cargo-cgp/) · [Installation](https://contextgeneric.dev/docs/cargo-cgp/installation) · [cargo-cgp repository](https://github.com/contextgeneric/cargo-cgp)**

## License

Licensed under the MIT license.
