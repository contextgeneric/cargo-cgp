//! A `#[use_provider]` bound naming the consumer trait of a component with a lifetime parameter.
//!
//! `#[use_provider(Inner: CanRead<'a, Source>)]` should name `Reader<'a, Source>`. The attribute
//! inserts the context after the lifetime, generating `Inner: CanRead<'a, __Context__, Source>`,
//! so the consumer trait receives one type argument too many and rustc reports `E0107`. The bound
//! sits on the inner provider with the context as its first type argument, the shape the attribute
//! writes, so the tool reshapes it into `[CGP-E034]` as it does for a component without lifetimes.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Reader)]
pub trait CanRead<'a, Source> {
    fn read(&self, source: &'a Source) -> &'a str;
}

// MISTAKE: `#[use_provider]` names the consumer trait `CanRead` where the provider trait `Reader`
// belongs.
#[cgp_impl(new ReadThrough<Inner>)]
#[use_provider(Inner: CanRead<'a, Source>)]
impl<'a, Inner, Source> Reader<'a, Source> {
    fn read(&self, source: &'a Source) -> &'a str {
        Inner::read(self, source)
    }
}

fn main() {}
