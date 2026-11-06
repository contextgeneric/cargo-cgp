//! A builder's `build_field` given a value of the wrong type for the field.
//!
//! The call names `first_name`, a `String` field, and passes a `u32`, so rustc reports an `E0308` on
//! the value. The field is still unset, so no field state explains the mismatch, and the tool
//! leaves the error as rustc wrote it, with its conversion suggestion.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Person::builder().build_field(PhantomData::<Symbol!("first_name")>, 42_u32);
}
