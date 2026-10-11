//! Two distinct types with the same name, which only their module paths tell apart.
//!
//! Plain `cargo check` reports "expected `a::Foo`, found `b::Foo`" and notes that the two have
//! similar names but are distinct types. The tool must print the same; its module strip rewrites the
//! paths away, leaving "expected `Foo`, found `Foo`".
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@rewrites-falsely: the module strip removes the paths that tell the two types apart

mod a {
    pub struct Foo;
}

mod b {
    pub struct Foo;
}

fn same_name() -> a::Foo {
    b::Foo
}

fn main() {
    let _ = same_name();
}
