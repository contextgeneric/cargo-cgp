//! Taking a record field out of an empty builder.
//!
//! `take_field` on a fresh builder fails as an `E0271` on `Mapper == IsPresent` for
//! `__PartialPerson<IsNothing, IsNothing>`. The output never says `first_name` has not been set.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::core::field::traits::TakeField;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Person::builder().take_field(PhantomData::<Symbol!("first_name")>);
}
