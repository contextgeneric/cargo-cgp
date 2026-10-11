//! A direct `update_field` call that leaves the target marker to inference.
//!
//! The error is plain Rust: type annotations are needed (`E0284`). The tool must print what plain
//! `cargo check` prints, keeping `UpdateField::update_field` in a label and the unlabelled span on
//! the method's definition.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Person::builder().update_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
}
