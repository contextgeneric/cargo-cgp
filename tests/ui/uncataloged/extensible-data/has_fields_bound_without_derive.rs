//! A provider bound on the context's `HasFields` shape, wired on a context without the derive.
//!
//! The check resolves to a `[CGP-E117]` leaf `App: HasFields` with no `#[derive(HasFields)]` help,
//! unlike the `HasField` leaves, which carry their derive help.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E117

use cgp::prelude::*;

#[cgp_component(Describer)]
pub trait CanDescribe {
    fn describe(&self) -> String;
}

#[cgp_impl(new DescribeNamed)]
#[uses(HasFields<Fields = Struct! { name: String }>)]
impl Describer {
    fn describe(&self) -> String {
        "named".into()
    }
}

#[derive(HasField)]
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        DescriberComponent: DescribeNamed,
    }
}

check_components! {
    App {
        DescriberComponent,
    }
}

fn main() {}
