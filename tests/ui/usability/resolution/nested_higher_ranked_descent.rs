//! Usability failure: the resolver declines a wiring failure that threads the record
//! field-list machinery through *nested* higher-ranked (`for<'a>`) hops, falling back to
//! raw rustc output.
//!
//! This is the faithful shape of `cgp-serde`'s `MessagesArchive` check. Encoding a record
//! walks its `HasFields` list (`EncodeFields` over `Cons`/`Field`/`Nil`), each field
//! encoded through the context; a `Vec` field routes through `EncodeIterator`, whose
//! higher-ranked dependency `Self: for<'a> CanEncodeValue<<&'a Value as IntoIterator>::Item>`
//! carries a projection through the bound lifetime; and the borrowed item routes back to
//! the owned value through `EncodeDeref`. `Outer` nests this twice — `Outer` → `Vec<Inner>`
//! → `Inner` → `Vec<u64>` → `u64` — and `u64` is never wired.
//!
//! Neither piece alone declines: the single-hop `higher_ranked_descent` and the record-only
//! `record_field_chain` both resolve cleanly. Only their combination — the record-list
//! recursion crossing two nested higher-ranked projection hops — makes the walk reach no
//! leaf, so `resolve_leaves` returns `None` and the diagnostic falls back to the text
//! rewrite, leaking the raw `IsProviderFor` / `__Context__` scaffolding and the `for<'a> _:
//! CanEncodeValue<…>` bound instead of the compact root-cause tree.

use cgp::prelude::*;

#[cgp_component(ValueEncoder)]
pub trait CanEncodeValue<Value: ?Sized> {
    fn encode_value(&self, value: &Value) -> String;
}

#[cgp_impl(new EncodeU64)]
impl ValueEncoder<u64> {
    fn encode_value(&self, value: &u64) -> String {
        value.to_string()
    }
}

#[cgp_impl(new EncodeIterator)]
impl<Value> ValueEncoder<Value>
where
    for<'a> &'a Value: IntoIterator,
    Self: for<'a> CanEncodeValue<<&'a Value as IntoIterator>::Item>,
{
    fn encode_value(&self, _value: &Value) -> String {
        String::new()
    }
}

#[cgp_impl(new EncodeDeref)]
#[uses(CanEncodeValue<Value>)]
impl<'a, Value> ValueEncoder<&'a Value> {
    fn encode_value(&self, value: &&'a Value) -> String {
        self.encode_value(*value)
    }
}

// A recursive field-list handler over the `Cons`/`Nil` spine: encodes the head field
// *through the context* and recurses on the tail (mirrors `cgp-serde`'s `FieldsSerializer`).
pub trait EncodeFields<Context, Value> {
    fn encode_fields(context: &Context, value: &Value) -> String;
}

impl<Context, Value, Tag, FieldValue, Tail> EncodeFields<Context, Value>
    for Cons<Field<Tag, FieldValue>, Tail>
where
    Context: CanEncodeValue<FieldValue>,
    Tail: EncodeFields<Context, Value>,
{
    fn encode_fields(_context: &Context, _value: &Value) -> String {
        String::new()
    }
}

impl<Context, Value> EncodeFields<Context, Value> for Nil {
    fn encode_fields(_context: &Context, _value: &Value) -> String {
        String::new()
    }
}

#[cgp_impl(new EncodeRecord)]
impl<Value> ValueEncoder<Value>
where
    Value: HasFields,
    Value::Fields: EncodeFields<Self, Value>,
{
    fn encode_value(&self, _value: &Value) -> String {
        String::new()
    }
}

#[derive(HasFields)]
pub struct Inner {
    pub value: u64,
}

#[derive(HasFields)]
pub struct Mid {
    pub inners: Vec<Inner>,
}

#[derive(HasFields)]
pub struct Outer {
    pub mids: Vec<Mid>,
}

pub struct App;

delegate_components! {
    App {
        open ValueEncoderComponent;

        @ValueEncoderComponent.[Outer, Mid, Inner]: EncodeRecord,
        @ValueEncoderComponent.[Vec<Mid>, Vec<Inner>]: EncodeIterator,
        @ValueEncoderComponent.<'a, T> &'a T: EncodeDeref,
        // `u64` is deliberately left unwired — the mistake this fixture pins, reached only
        // through three record layers and two nested higher-ranked iterator hops.
    }
}

check_components! {
    App {
        ValueEncoderComponent: [Outer],
    }
}

fn main() {}
