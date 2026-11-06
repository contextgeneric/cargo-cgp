//! `#[uses(Reader<'static, Text>)]` on a `#[cgp_impl]` provider, naming the provider trait of a
//! component with a lifetime where its consumer trait belongs.
//!
//! The provider trait `Reader<'a, Context, Source>` takes the context after the lifetime, so the
//! type argument written fills the context's place and rustc reports `E0107`. The tool reshapes it
//! into `[CGP-E037]` as it does for a component without parameters, counting only the type
//! arguments, and its help names `#[uses(CanRead<'static, Text>)]`.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E037

use cgp::prelude::*;

#[cgp_component(Reader)]
pub trait CanRead<'a, Source> {
    fn read(&self, source: &'a Source) -> &'a str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct Text;

#[cgp_impl(new GreetByReading)]
#[uses(Reader<'static, Text>)]
impl Greeter {
    fn greet(&self) -> String {
        self.read(&Text).to_owned()
    }
}

fn main() {}
