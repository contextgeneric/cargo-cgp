//! A direct entry for `@DescriberComponent.Wrap<u64>` beside a later entry that redirects the
//! overlapping `@DescriberComponent.<T> Wrap<T>`. The two `DelegateComponent<PathCons<..>>` impls
//! conflict with the coherence error E0119 over `Wrap<u64>`.
//!
//! The two entries map different keys, so the tool words the pair as an overlap,
//! `[CGP-E011] conflicting wiring: `App` wires `@DescriberComponent.Wrap<u64>`, which
//! `@DescriberComponent.Wrap<T>` already covers`, rather than as a key wired directly beside its
//! own redirect ([open_prefixed_component.rs](open_prefixed_component.rs)).
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).

//@quality: acceptable
//@codes: CGP-E011

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
