[![Crates.io](https://img.shields.io/crates/v/cargo-cgp-expand.svg)](https://crates.io/crates/cargo-cgp-expand)
[![Docs.rs](https://docs.rs/cargo-cgp-expand/badge.svg)](https://docs.rs/cargo-cgp-expand)

# cargo-cgp-expand: the compiler-free half of `cargo cgp expand`

**The library that rewrites CGP's type-level constructs in expanded source back to the macros the
programmer wrote, and narrows an expansion to one module or item.**

It backs `cargo cgp expand`, the [`cargo-cgp`](https://crates.io/crates/cargo-cgp) command that shows
the Rust a crate's CGP macros generate. A field tag comes back as `Symbol!("height")` rather than the
raw `Symbol<6, Chars<…>>` type. It links no compiler internals, so it builds and tests on any
toolchain.

The crate is published so that `cargo-cgp-driver` can depend on it, and its API serves that driver
alone, so it may change in any release. To expand CGP code, install `cargo-cgp` instead.

**[Website](https://contextgeneric.dev/docs/cargo-cgp/expand) · [cargo-cgp repository](https://github.com/contextgeneric/cargo-cgp)**

## License

Licensed under the MIT license.
