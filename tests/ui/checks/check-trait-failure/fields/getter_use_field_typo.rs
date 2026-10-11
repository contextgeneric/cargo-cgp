//! `UseField<Symbol!("frist_name")>` for a field named `first_name`.
//!
//! `[CGP-E106] missing field `frist_name`` is right, but rustc's near-impl hint naming `first_name`
//! is dropped, so the typo is not pointed out.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E106

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
