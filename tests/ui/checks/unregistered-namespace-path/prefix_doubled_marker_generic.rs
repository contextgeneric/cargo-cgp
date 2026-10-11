//! A prefix that repeats the component marker the path already appends.
//!
//! The doubled marker is visible in the `[CGP-E107]` path but not called out.
//!
//! CGP error class: unregistered namespace path (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/unregistered-namespace-path.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E104 CGP-E107

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
