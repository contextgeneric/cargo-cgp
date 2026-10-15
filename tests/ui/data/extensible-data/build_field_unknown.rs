//! Building a field the record does not declare.
//!
//! `build_field` for `age` on a `Person` without that field has no `UpdateField` impl to use. The
//! data-call anchor names the field: ``[CGP-E050] missing field `age` on `Person` ``, with no
//! `help`, since the tag may be the mistake instead.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E110

use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _ = Person::builder().build_field(PhantomData::<Symbol!("age")>, 42_u8);
}
