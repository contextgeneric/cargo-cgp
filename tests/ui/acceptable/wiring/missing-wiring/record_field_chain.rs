//! Acceptable: a missing wiring reached only *through* the extensible-data record machinery — a
//! record provider that builds each field through the context, over a recursive `Cons`/`Nil`
//! field-list handler. This is the shape `cgp-serde`'s arena test hits: checking `Payload`
//! deserialization descends `DeserializeRecordFields` → `HandleMapEntry` over the field list → a
//! field whose value type (`&Coord`) the context never wires.
//!
//! The chain exercises three things the walk must handle together, none of which a flat wiring
//! failure needs:
//!
//!  - **A provider that also matches the delegation blanket.** `BuildRecord: ValueBuilder<App, Outer>`
//!    unifies with both the CGP blanket `impl<P: DelegateComponent> ValueBuilder for P` and its own
//!    `#[cgp_impl]`; the walk must prefer the concrete-`Self` impl, or it dead-ends on
//!    `BuildRecord: DelegateComponent`.
//!  - **An associated-type-determined provider parameter.** `BuildRecord`'s `Builder` param is fixed
//!    only by `Value: HasRecordBuilder<RecordBuilder = Builder>`; the walk must solve that clause to
//!    bind `Builder` before the sibling `Value::Fields: BuildFields<Self, Builder>` clause — the
//!    branch that leads to the cause — reads as anything but a stray inference var.
//!  - **A same-trait recursion over a type-level list.** `BuildFields` handles its head field here and
//!    its later fields through the tail `Cons<.., Nil>: BuildFields<..>`, a same-trait bound on
//!    another foreign list node; the walk must follow that recursion to reach the field whose
//!    dependency is unwired.
//!
//! The single mistake is the un-wired `@ValueBuilderComponent.Inner`; the root cause names exactly
//! that path, reached through the record chain.
//!
//! See cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md (the walk).

use cgp::prelude::*;

// The per-value build capability, dispatched per value type with `open`.
#[cgp_component(ValueBuilder)]
pub trait CanBuildValue<Value> {
    fn build_value(&self) -> Value;
}

// A scalar leaf provider.
#[cgp_impl(new BuildU64)]
impl ValueBuilder<u64> {
    fn build_value(&self) -> u64 {
        0
    }
}

// Each record names its own builder type — the associated-type-determined `Builder` param the
// record provider carries (mirrors `HasOptionalBuilder<Builder = …>`).
pub trait HasRecordBuilder {
    type RecordBuilder: Default;
}

// A recursive field-list handler over the `Cons`/`Nil` spine: builds the head field *through the
// context* and recurses on the tail (mirrors `HandleMapEntry`).
pub trait BuildFields<Context, Builder> {
    fn build_fields(context: &Context) -> Builder;
}

impl<Context, Tag, Value, Tail, Builder> BuildFields<Context, Builder>
    for Cons<Field<Tag, Value>, Tail>
where
    Context: CanBuildValue<Value>,
    Tail: BuildFields<Context, Builder>,
{
    fn build_fields(context: &Context) -> Builder {
        Tail::build_fields(context)
    }
}

impl<Context, Builder: Default> BuildFields<Context, Builder> for Nil {
    fn build_fields(_context: &Context) -> Builder {
        Builder::default()
    }
}

// The record provider: its `Builder` param is fixed by the `HasRecordBuilder` clause, then flows
// into the recursive field-list handler.
#[cgp_impl(new BuildRecord)]
impl<Value, Builder> ValueBuilder<Value>
where
    Value: HasFields + HasRecordBuilder<RecordBuilder = Builder>,
    Value::Fields: BuildFields<Self, Builder>,
{
    fn build_value(&self) -> Value {
        todo!()
    }
}

#[derive(HasFields)]
pub struct Inner {
    pub value: u64,
}

#[derive(HasFields)]
pub struct Outer {
    pub id: u64,
    pub inner: Inner,
}

impl HasRecordBuilder for Inner {
    type RecordBuilder = ();
}

impl HasRecordBuilder for Outer {
    type RecordBuilder = ();
}

pub struct App;

delegate_components! {
    App {
        open ValueBuilderComponent;

        @ValueBuilderComponent.u64: BuildU64,
        @ValueBuilderComponent.Outer: BuildRecord,
        // `Inner` is deliberately left unwired — the mistake this fixture pins.
        // @ValueBuilderComponent.Inner: BuildRecord,
    }
}

check_components! {
    App {
        ValueBuilderComponent: [
            Outer,
        ],
    }
}

fn main() {}
