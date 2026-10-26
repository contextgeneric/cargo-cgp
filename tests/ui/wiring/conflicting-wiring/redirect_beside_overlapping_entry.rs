//! A direct entry for `@DescriberComponent.Wrap<u64>` beside a later entry that redirects the
//! overlapping `@DescriberComponent.<T> Wrap<T>`. The two `DelegateComponent<PathCons<..>>` impls
//! conflict with the coherence error E0119 over `Wrap<u64>`.
//!
//! One entry redirects, so the tool reads the pair as a redirect collision on one key and reports
//! `App` wiring `@DescriberComponent.Wrap<T>` directly, which is false: that entry is the redirect.
//!
//! The overlapping counterpart of [open_prefixed_component.rs](open_prefixed_component.rs), whose
//! two entries map one key.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@quality: usability
//@codes: CGP-E013

use cgp::prelude::*;

#[cgp_component(Describer)]
pub trait CanDescribe<T> {
    fn describe(&self, value: &T) -> String;
}

#[cgp_impl(new DescribeAny)]
impl<T> Describer<T> {
    fn describe(&self, _value: &T) -> String {
        "a value".to_owned()
    }
}

pub struct Wrap<T>(pub T);

pub struct App;

delegate_components! {
    App {
        @DescriberComponent.Wrap<u64>: DescribeAny,
        @DescriberComponent.<T> Wrap<T> => @app.describer,
    }
}

fn main() {}
