//! Merging a source record with a field the target does not declare.
//!
//! `Person { first_name, surname }` merged into `Employee { first_name, last_name }` fails on the
//! private `FieldsBuilder` recursion over the source's fields, which the chain drops:
//! ``[CGP-E050] missing field `surname` on `Employee` ``, under the `BuildField` step that names
//! the field.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E110

use cgp::core::field::impls::CanBuildFrom;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Person {
    pub first_name: String,
    pub surname: String,
}

#[derive(CgpData)]
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
}

fn main() {
    let _: Employee = Employee::builder()
        .build_from(Person { first_name: "A".into(), surname: "B".into() })
        .finalize_build();
}
