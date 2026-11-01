//! A `macro_rules!` call matching no rule stops the compilation once expansion finishes.
//!
//! rustc aborts when a macro's expansion failed, so `cargo cgp expand` has nothing to print, its
//! `.expand.rs` holds the marker, and the command fails with cargo's code.
//!
//! CGP error class: none; plain Rust the tool passes through.

macro_rules! only_ok {
    (ok) => {};
}

only_ok!(bad);

fn main() {}
