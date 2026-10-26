//! An `open` key for one instance of a generic type, followed by a key whose segment is the generic
//! type: `@DescriberComponent.Wrap<u64>`, then `@DescriberComponent.<T> Wrap<T>`, which covers it.
//! The two `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119.
//!
//! The tool reports `App` wiring `@DescriberComponent.Wrap<T>`, "which
//! `@DescriberComponent.Wrap<u64>` already covers", which is false: the generic key covers the
//! specific one, not the reverse. The wording follows the order of the entries.
//!
//! The reversed counterpart of
//! [path_generic_type_segment_overlap.rs](path_generic_type_segment_overlap.rs).
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@quality: usability
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
