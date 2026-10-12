//! An `open` key whose segment is a generic type, overlapping a key for one of its instances.
//! `@DescriberComponent.<T> Wrap<T>` covers every `Wrap<…>`, so it overlaps the
//! `@DescriberComponent.Wrap<u64>` entry beside it, and the two `DelegateComponent<PathCons<..>>`
//! impls conflict with the coherence error E0119.
//!
//! The tool names each type segment by its type alone, dropping its arguments, so the two keys
//! render the same and the overlap is misreported as `[CGP-E010] duplicate wiring for
//! `@DescriberComponent.Wrap` on `App``, a statement the program contradicts.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).

//@quality: usability
//@codes: CGP-E010

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
