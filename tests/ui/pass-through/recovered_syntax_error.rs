//! A syntax error the parser recovers from leaves the rest of the crate to expand.
//!
//! The parser reports the missing expression and goes on, so `cargo cgp expand` prints the listing
//! and still fails with cargo's code, unlike
//! [`unclosed_delimiter`](./unclosed_delimiter.rs), which stops the compilation.
//!
//! CGP error class: none; plain Rust the tool passes through.

pub struct Kept;

fn main() {
    let _value = ;
}
