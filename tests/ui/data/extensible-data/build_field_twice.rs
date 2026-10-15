//! Building the same record field twice, which the builder's type rejects.
//!
//! The second `build_field` for `first_name` fails on the `UpdateField` bound behind `BuildField`,
//! whose `Mapper` finds the field already set. The data-call anchor types the builder through the
//! chain of `builder()` and `build_field` calls and reads its presence markers:
//! ``[CGP-E050] field `first_name` of `Person` is already set``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E118

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
