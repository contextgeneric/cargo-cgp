//! Merging a source record with a field the target does not declare.
//!
//! `Person { first_name, surname }` merged into `Employee { first_name, last_name }` fails on the
//! private `FieldsBuilder` recursion over the remaining `Struct! { surname: String }`, with a long
//! help pointing into `cgp-field`. The output never says `Employee` has no `surname` field.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
