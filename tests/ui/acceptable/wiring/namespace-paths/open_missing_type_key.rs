//! Acceptable: an `open`-dispatched component checked for a value type the context never wires. The
//! redirect leaf names the missing path (`@ItemEncoderComponent.Vec<u8>`), the same way the redirect
//! hop above it names `@ItemEncoderComponent`, so the reader sees exactly what to wire.
//!
//! This is the shape `cgp-serde`'s arena test hits: `open ValueDeserializerComponent` with
//! `@ValueDeserializerComponent.<'b> &'b Coord: DeserializeAndAllocate` commented out, so checking
//! `&'a Coord` must name `@ValueDeserializerComponent.&'a Coord` rather than a bare `PathCons`.
//!
//! Two things had to line up. The missing `DelegateComponent` key is a redirect *path*
//! (`PathCons<ItemEncoderComponent, PathCons<Vec<u8>, Nil>>`), not a bare component marker, so the
//! leaf classifier renders the whole path (as a [missing-redirect-wiring
//! leaf](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/implementation/typed-root-cause-resolution.md))
//! rather than reading only its ADT item name (`PathCons`); and the `Path!` resugarer renders the
//! value segment `Vec<u8>` verbatim rather than declining it for carrying generics, so the path
//! folds to `@ItemEncoderComponent.Vec<u8>`.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md (the
//! missing-redirect-wiring leaf).

use cgp::prelude::*;

#[cgp_component(ItemEncoder)]
pub trait CanEncodeItem<Value> {
    fn encode_item(&self, value: &Value) -> String;
}

#[cgp_impl(new EncodeDisplay)]
impl<Value> ItemEncoder<Value>
where
    Value: core::fmt::Display,
{
    fn encode_item(&self, value: &Value) -> String {
        value.to_string()
    }
}

pub struct App;

delegate_components! {
    App {
        open ItemEncoderComponent;

        @ItemEncoderComponent.u64: EncodeDisplay,
        // `Vec<u8>` is deliberately left unwired — the mistake this fixture pins.
        // @ItemEncoderComponent.Vec<u8>: EncodeBytes,
    }
}

check_components! {
    App {
        ItemEncoderComponent: [
            u64,
            Vec<u8>,
        ],
    }
}

fn main() {}
