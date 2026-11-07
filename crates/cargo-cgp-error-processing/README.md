[![Crates.io](https://img.shields.io/crates/v/cargo-cgp-error-processing.svg)](https://crates.io/crates/cargo-cgp-error-processing)
[![Docs.rs](https://docs.rs/cargo-cgp-error-processing/badge.svg)](https://docs.rs/cargo-cgp-error-processing)

# cargo-cgp-error-processing: the compiler-free half of cargo-cgp's diagnostics

**The library that holds the text rewrites, the root-cause wording, and the dependency-tree rendering
that `cargo-cgp-driver` applies to CGP compiler errors.**

It links no compiler internals, so it builds and tests on any toolchain.

The crate is published so that `cargo-cgp-driver` can depend on it, and its API serves that driver
alone, so it may change in any release. To read CGP errors, install `cargo-cgp` instead.

**[Website](https://contextgeneric.dev/docs/cargo-cgp/reading-output) · [Error codes](https://contextgeneric.dev/docs/cargo-cgp/error-codes) · [cargo-cgp repository](https://github.com/contextgeneric/cargo-cgp)**

## License

Licensed under the MIT license.
