//! Use-site failure on an `open`-dispatched context, resolved to the real missing wiring.
//!
//! `App` dispatches `ItemEncoderComponent` per value type with an `open` statement. It wires
//! `Seq<u64>` to `EncodeSeq`, which encodes a sequence by encoding each item, so it depends on
//! `Self: CanEncodeItem<u64>` — but `App` never wires `u64`. Calling `app.encode_item(&Seq(..))`
//! therefore fails at the use site with an unsatisfied transitive dependency, and there is no
//! `check_components!` entry to anchor on.
//!
//! The use-site resolver recovers `App` and re-checks every component it wires, read from its
//! `DelegateComponent` impls. For an `open`-dispatched context those impls are per-value redirect
//! entries whose keys are `PathCons<…>` *paths*, not component markers. The resolver recovers the
//! real dispatch parameter from each two-segment path (`@ItemEncoderComponent.Seq<u64>` →
//! `CanUseComponent<ItemEncoderComponent, Seq<u64>>`) rather than re-checking the raw `PathCons`
//! key — which would report the type-level `PathCons` spine as a bogus "consumer trait" and bottom
//! out on `T: Sized` noise — so the failure is traced to the real gap: the unwired `u64` encoder.

use cgp::prelude::*;

#[cgp_component(ItemEncoder)]
pub trait CanEncodeItem<Value> {
    fn encode_item(&self, value: &Value) -> String;
}

pub struct Seq<T>(pub Vec<T>);

#[cgp_impl(new EncodeSeq)]
#[uses(CanEncodeItem<Item>)]
impl<Item> ItemEncoder<Seq<Item>> {
    fn encode_item(&self, value: &Seq<Item>) -> String {
        let mut out = String::new();
        for item in value.0.iter() {
            out.push_str(&self.encode_item(item));
        }
        out
    }
}

pub struct App;

delegate_components! {
    App {
        open ItemEncoderComponent;

        @ItemEncoderComponent.Seq<u64>: EncodeSeq,
    }
}

fn main() {
    let app = App;
    let _ = app.encode_item(&Seq(vec![1u64]));
}
