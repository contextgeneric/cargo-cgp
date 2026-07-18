//! Usability failure — currently a compiler **panic**, not a bad message: the
//! resolver's dependency walk crashes rustc when it descends into a higher-ranked
//! (`for<'a>`) CGP obligation.
//!
//! `EncodeSeq` serializes a `Seq<Value>` by encoding each borrowed item, so it depends
//! on `Self: for<'a> CanEncodeItem<&'a Value>` — a higher-ranked impl-side dependency,
//! the same shape `cgp-serde`'s `SerializeIterator` carries (`Self: for<'a>
//! CanSerializeValue<<&'a Value as IntoIterator>::Item>`). `App` wires `Seq<u64>` to
//! `EncodeSeq` but never wires an encoder for the borrowed `&u64` the sequence yields,
//! so the `check_components!` entry for `Seq<u64>` fails.
//!
//! While resolving that failure, the walk descends to the unmet `App: for<'a>
//! CanEncodeItem<&'a u64>` obligation. To find the impl that would satisfy it, the
//! resolver calls `ocx.eq` on the obligation's trait ref — but reaches it through
//! `skip_binder()`, which leaves the `'a` bound variable escaping. rustc's inference
//! generalizer asserts `!source_term.has_escaping_bound_vars()`, and the compiler
//! panics with an ICE instead of reporting the missing wiring.
//!
//! The fix instantiates the obligation's binder with fresh inference variables before
//! the `eq`, so the walk descends past the higher-ranked bound and bottoms out on the
//! real root cause — the missing `@ItemEncoderComponent.&u64` wiring — with no panic.

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
