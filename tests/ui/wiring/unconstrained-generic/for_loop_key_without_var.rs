//! A `for` loop whose key does not mention the loop variable.
//!
//! rustc's three `E0207`s, with contradictory helps, become one `[CGP-E017]` naming the variable.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Its help, writing `T` in the key, compiles; the caret stays on the variable rustc named first.

//@quality: acceptable
//@codes: CGP-E017

use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@test in DefaultNamespace)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
#[default_impl(String in DefaultImpls1<ShowImplComponent>)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        for <T, Provider> in DefaultImpls1<ShowImplComponent> {
            @test.ShowImplComponent.String: Provider,
        }
    }
}

fn main() {}
