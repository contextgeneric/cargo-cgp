# cargo-cgp-error-processing

`cargo-cgp-error-processing` holds the compiler-free half of
[`cargo-cgp`](https://crates.io/crates/cargo-cgp)'s diagnostic transforms: the text rewrites, the
root-cause wording, and the dependency-tree rendering that `cargo-cgp-driver` applies to CGP
compiler errors. It links no compiler internals, so it builds and tests on any toolchain.

It is published so that `cargo-cgp-driver` can depend on it, and its API serves that driver alone:
it may change in any release. To read CGP errors, install `cargo-cgp` instead.

See the [`cargo-cgp` repository](https://github.com/contextgeneric/cargo-cgp) for how the tool
works.
