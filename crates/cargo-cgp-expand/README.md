# cargo-cgp-expand

`cargo-cgp-expand` holds the compiler-free half of `cargo cgp expand`, the
[`cargo-cgp`](https://crates.io/crates/cargo-cgp) command that shows the Rust a crate's CGP macros
generate. It rewrites CGP's type-level constructs in the expanded source back to the macros the
programmer wrote, such as `Symbol!("height")`, and narrows an expansion to one module or item. It
links no compiler internals, so it builds and tests on any toolchain.

It is published so that `cargo-cgp-driver` can depend on it, and its API serves that driver alone:
it may change in any release. To expand CGP code, install `cargo-cgp` instead.

See the [`cargo-cgp` repository](https://github.com/contextgeneric/cargo-cgp) for how the tool
works.
