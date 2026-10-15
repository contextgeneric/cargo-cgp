//! Taking a record field out of an empty builder.
//!
//! `take_field` on a fresh builder fails on the `UpdateField` bound behind `TakeField`, whose
//! `Mapper` finds the field unset: ``[CGP-E050] field `first_name` of `Person` is not set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E119

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
