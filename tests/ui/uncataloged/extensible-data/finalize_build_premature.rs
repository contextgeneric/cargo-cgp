//! Finalizing a record builder before every field is set.
//!
//! `finalize_build` with `last_name` unset is an `E0599` "no method named `finalize_build`", labelled
//! with the partial type `__PartialPerson<IsPresent, IsNothing>`. The marker list is the only clue;
//! the unset field is never named.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
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
    let _: Person = Person::builder()
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
        .finalize_build();
}
