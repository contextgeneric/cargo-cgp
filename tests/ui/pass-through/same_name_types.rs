//! Two distinct types with the same name, which only their module paths tell apart.
//!
//! Plain `cargo check` reports "expected `a::Foo`, found `b::Foo`" and notes that the two have
//! similar names but are distinct types. The tool must print the same, keeping the paths that tell
//! the two types apart.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

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
