//! Building the same record field twice, which the builder's type rejects.
//!
//! The second `build_field` for `first_name` fails on the `UpdateField` bound behind `BuildField`, as an
//! `E0271` on `Mapper == IsNothing` with a note on the derive. The output never says the field is
//! already set; the first `IsPresent` in `__PartialPerson<IsPresent, IsNothing>` is the only sign.
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
    let _ = Person::builder()
        .build_field(PhantomData::<Symbol!("first_name")>, "Alice".to_owned())
        .build_field(PhantomData::<Symbol!("first_name")>, "Bob".to_owned());
}
