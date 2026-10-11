//! A hand-written `FieldGetter` wired directly as a component's provider.
//!
//! `[CGP-E111]` is accurate; the fix, `WithProvider<…>`, is not hinted.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E002 CGP-E101 CGP-E102 CGP-E111

use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

pub struct Profile {
    pub display_name: String,
}

#[derive(HasField)]
pub struct Account {
    pub profile: Profile,
}

pub struct ReadDisplayName;

impl<Tag> FieldGetter<Account, Tag> for ReadDisplayName {
    type Value = String;

    fn get_field(context: &Account, _tag: PhantomData<Tag>) -> &String {
        &context.profile.display_name
    }
}

delegate_components! {
    Account {
        NameGetterComponent: ReadDisplayName,
    }
}

check_components! {
    Account {
        NameGetterComponent,
    }
}

fn main() {}
