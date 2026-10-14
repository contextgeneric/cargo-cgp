//! Building a field the record does not declare.
//!
//! `build_field` for `age` on a `Person` without that field is an `E0277` on
//! `UpdateField<Symbol!("age"), IsPresent>`, followed by the impls that do exist, one per declared
//! field. The output never states that `Person` has no field `age`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Person::builder().build_field(PhantomData::<Symbol!("age")>, 42_u8);
}
