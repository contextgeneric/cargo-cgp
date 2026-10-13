//! A `#[cgp_computer]` function whose type parameter `T` appears only in its return type, which the
//! macro rejects at expansion.
//!
//! The generated `Computer` impl's trait arguments carry only the input types, so nothing would
//! determine `T`; the macro says so on the `T` the user wrote. The tool must print the macro's
//! message as plain `cargo check` prints it.
//!
//! CGP error class: none; a macro's own rejection, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_computer.md.

use cgp::prelude::*;

#[cgp_computer]
fn parse<T: core::str::FromStr>(value: String) -> Option<T> {
    value.parse().ok()
}

fn main() {}
