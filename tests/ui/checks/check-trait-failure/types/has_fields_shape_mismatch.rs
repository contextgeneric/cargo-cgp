//! A provider bound on the context's whole shape, wired on a context whose field type differs.
//!
//! `DescribeNamed` requires `HasFields<Fields = Struct! { name: String }>` and `App` has `name: u32`.
//! The projection mismatch becomes `[CGP-E005]` naming the associated type `Fields` of `HasFields`,
//! with both shapes resugared, over a tree ending in the `[CGP-E116]` leaf.
//!
//! CGP error class: check-trait failure, its projection-mismatch face
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E005 CGP-E101 CGP-E102 CGP-E116

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

#[derive(HasFields)]
pub struct App {
    pub name: u32,
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
