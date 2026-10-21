//! A crate's own macro whose name ends in `check_components` defines one item twice.
//!
//! `table_check_components!` expands to two traits named `Dup`, an `E0428` whose span comes from a
//! bang macro of that name. The tool reshapes an `E0428` from CGP's `check_components!` into
//! `[CGP-E016]`; this macro is not CGP's, so its error must stay exactly as rustc prints it.
//!
//! CGP error class: none; plain Rust the tool passes through.

//@rewrites-falsely: the `[CGP-E016]` reshape matches any bang macro whose name ends in `check_components`

macro_rules! table_check_components {
    () => {
        pub trait Dup {}
        pub trait Dup {}
    };
}

table_check_components!();

fn main() {}
