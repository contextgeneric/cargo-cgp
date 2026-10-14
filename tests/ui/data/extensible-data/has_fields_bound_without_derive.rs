//! A provider bound on the context's `HasFields` shape, wired on a context without the derive.
//!
//! The check resolves to the unmet `App: HasFields`, which the derive supplies:
//! ``[CGP-E001] `App` does not implement `HasFields` ``, with a help naming `#[derive(HasFields)]`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E123

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
