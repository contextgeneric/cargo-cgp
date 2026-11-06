//! A prefix that repeats the component marker the path already appends.
//!
//! The doubled marker shows in the `[CGP-E113]` path, and a `help` names the near-miss entry.
//!
//! CGP error class: unregistered namespace path (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/unregistered-namespace-path.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E103 CGP-E113

use cgp::prelude::*;

#[cgp_component(ShowImpl)]
#[prefix(@app.ShowImplComponent in DefaultNamespace)]
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowString)]
impl ShowImpl<String> {
    fn show(&self, value: &String) -> String {
        value.clone()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @app.ShowImplComponent.String: ShowString,
    }
}

check_components! {
    App {
        ShowImplComponent: String,
    }
}

fn main() {}
