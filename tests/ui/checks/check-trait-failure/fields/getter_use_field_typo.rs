//! `UseField<Symbol!("frist_name")>` for a field named `first_name`.
//!
//! The headline reads `` [CGP-E001] missing field `frist_name` on `Person` ``, with the same
//! text as the `` [CGP-E110] `` root cause, and a `help` names the similar field `first_name`,
//! so the typo is pointed out.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[derive(HasField)]
pub struct Person {
    pub first_name: String,
}

delegate_components! {
    Person {
        NameGetterComponent: UseField<Symbol!("frist_name")>,
    }
}

fn main() {
    let _ = Person { first_name: String::new() }.name();
}
