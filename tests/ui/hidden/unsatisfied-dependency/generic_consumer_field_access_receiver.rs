//! A generic consumer called on a field access. The use-site anchor reaches a component only with
//! the parameter its key carries, and this one rides in the call, whose receiver the call-site
//! anchor cannot type. So the resolver declines and the cause stays under rustc's headline.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Encoder)]
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> String;
}

#[cgp_impl(new EncodeNamed)]
impl<Value> Encoder<Value> {
    fn encode(&self, _value: &Value, #[implicit] name: &str) -> String {
        name.to_owned()
    }
}

#[derive(HasField)]
pub struct App {
    pub other: String,
}

delegate_components! {
    App {
        EncoderComponent: EncodeNamed,
    }
}

pub struct Holder {
    pub app: App,
}

fn main() {
    let holder = Holder { app: App { other: String::new() } };
    let _ = holder.app.encode(&1u32);
}
