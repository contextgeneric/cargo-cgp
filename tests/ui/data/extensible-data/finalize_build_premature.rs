//! Finalizing a record builder before every field is set.
//!
//! `finalize_build` with `last_name` unset is an `E0599` "no method named `finalize_build`", since
//! only the all-set builder implements `FinalizeBuild`. The data-call anchor reads the builder's
//! markers: ``[CGP-E050] field `last_name` of `Person` is not set``, under a chain that names the
//! finalize.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E119

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
