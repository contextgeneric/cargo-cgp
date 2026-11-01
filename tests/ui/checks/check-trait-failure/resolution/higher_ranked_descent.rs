//! A wiring failure through a higher-ranked obligation, where a careless walk would panic: the
//! resolver's dependency walk descends into a higher-ranked
//! (`for<'a>`) CGP obligation.
//!
//! `EncodeSeq` serializes a `Seq<Value>` by encoding each borrowed item, so it depends
//! on `Self: for<'a> CanEncodeItem<&'a Value>`, a higher-ranked impl-side dependency. `App` wires `Seq<u64>` to
//! `EncodeSeq` but never wires an encoder for the borrowed `&u64` the sequence yields,
//! so the `check_components!` entry for `Seq<u64>` fails.
//!
//! While resolving that failure, the walk descends to the unmet `App: for<'a>
//! CanEncodeItem<&'a u64>` obligation. To find the impl that would satisfy it, the
//! resolver relates the obligation's trait ref with `ocx.eq`. Reaching it through
//! `skip_binder()` would leave the `'a` bound variable escaping, and rustc's inference
//! generalizer asserts `!source_term.has_escaping_bound_vars()`, so the compiler
//! would panic instead of reporting the missing wiring.
//!
//! The walk instantiates the obligation's binder with placeholders before
//! the `eq`, so it descends past the higher-ranked bound and bottoms out on the
//! real root cause, the missing `@ItemEncoderComponent.&u64` wiring, with no panic.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E103 CGP-E113

use cgp::prelude::*;

#[cgp_component(ItemEncoder)]
pub trait CanEncodeItem<Value> {
    fn encode_item(&self, value: &Value) -> String;
}

pub struct Seq<T>(pub Vec<T>);

#[cgp_impl(new EncodeSeq)]
impl<Value> ItemEncoder<Seq<Value>>
where
    Self: for<'a> CanEncodeItem<&'a Value>,
{
    fn encode_item(&self, _value: &Seq<Value>) -> String {
        String::new()
    }
}

pub struct App;

delegate_components! {
    App {
        open ItemEncoderComponent;

        @ItemEncoderComponent.Seq<u64>: EncodeSeq,
    }
}

check_components! {
    App {
        ItemEncoderComponent: [Seq<u64>],
    }
}

fn main() {}
