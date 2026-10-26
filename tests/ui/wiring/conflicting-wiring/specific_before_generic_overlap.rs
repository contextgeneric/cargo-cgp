//! An `open` key for one instance of a generic type, followed by a key whose segment is the generic
//! type: `@DescriberComponent.Wrap<u64>`, then `@DescriberComponent.<T> Wrap<T>`, which covers it.
//! The two `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119.
//!
//! The tool names the generic key as the one that covers the other, whichever is written first:
//! `[CGP-E011] conflicting wiring: `App` wires `@DescriberComponent.Wrap<u64>`, which
//! `@DescriberComponent.Wrap<T>` already covers`, as for the reversed order in
//! [path_generic_type_segment_overlap.rs](path_generic_type_segment_overlap.rs).
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

        @DescriberComponent.Wrap<u64>: DescribeAny,
        @DescriberComponent.<T> Wrap<T>: DescribeAny,
    }
}

fn main() {}
