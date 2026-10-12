//! An `open` key whose segment is a generic type, overlapping a key for one of its instances.
//! `@DescriberComponent.<T> Wrap<T>` covers every `Wrap<…>`, so it overlaps the
//! `@DescriberComponent.Wrap<u64>` entry beside it, and the two `DelegateComponent<PathCons<..>>`
//! impls conflict with the coherence error E0119.
//!
//! The tool rewrites the conflict to `[CGP-E011] conflicting wiring: `App` wires
//! `@DescriberComponent.Wrap<u64>`, which `@DescriberComponent.Wrap<T>` already covers`: each type
//! segment keeps its
//! arguments, which is what tells the two keys apart. Dropping them would render both keys the
//! same and misreport the overlap as a duplicate.
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
        open DescriberComponent;

        @DescriberComponent.<T> Wrap<T>: DescribeAny,
        @DescriberComponent.Wrap<u64>: DescribeAny,
    }
}

fn main() {}
