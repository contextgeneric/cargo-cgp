//! A typo in the body of a `#[cgp_impl]` that names the consumer trait is dropped with the mistake.
//!
//! The consumer trait in the header is `[CGP-E032]`, and the header's error makes every name in the
//! body unresolved, so the tool drops the `E0425`s and the like in that body as its consequences. A
//! span is all that ties such an error to the mistake, so the misspelled `undefined_name`, the
//! program's own error, goes as well. It returns once the header is fixed.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md>.

//@quality: usability
//@codes: CGP-E032

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl CanGreet {
    fn greet(&self) {
        println!("{}", undefined_name);
    }
}

fn main() {}
