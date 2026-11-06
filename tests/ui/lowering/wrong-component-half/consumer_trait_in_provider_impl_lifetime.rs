//! A `#[cgp_impl]` header naming the consumer trait of a component with a lifetime parameter.
//!
//! The same mistake as `consumer_trait_in_provider_impl_generic`, on `CanRead<'a, Source>`. The
//! macro writes the header as `CanRead<'a, __Context__, Text>`, placing the context after the
//! lifetime, so rustc reports `E0107`. The tool reshapes it into `[CGP-E032]`, and the help writes
//! the provider trait with the arguments the header gave the consumer trait, `Reader<'a, Text>`,
//! without the inserted context.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E032

use cgp::prelude::*;

// A component with a lifetime and a type parameter: the consumer trait `CanRead<'a, Source>` pairs
// with the provider trait `Reader<'a, Context, Source>`.
#[cgp_component(Reader)]
pub trait CanRead<'a, Source> {
    fn read(&self, source: &'a Source) -> &'a str;
}

pub struct Text;

// MISTAKE: the `#[cgp_impl]` header names the consumer trait `CanRead` where the provider trait
// `Reader` belongs, so the consumer trait is given the inserted context as an extra argument.
#[cgp_impl(new ReadText)]
impl<'a> CanRead<'a, Text> {
    fn read(&self, _source: &'a Text) -> &'a str {
        ""
    }
}

fn main() {}
