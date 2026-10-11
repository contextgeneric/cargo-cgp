//! A direct `update_field` call that leaves the target marker to inference.
//!
//! The `E0284` headline reads `cgp::prelude::IsNothing` where plain `cargo check` prints `IsNothing`,
//! and the module strip shortens `UpdateField::update_field` to `update_field` in a label.
//!
//! CGP error class: none; a driver path-printing defect affecting every untransformed diagnostic.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Person::builder().update_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned());
}
