#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanBuildValue<Value> {
    fn build_value(&self) -> Value;
}
impl<__Context__, Value> CanBuildValue<Value> for __Context__
where
    __Context__: ValueBuilder<__Context__, Value>,
{
    fn build_value(&self) -> Value {
        __Context__::build_value(self)
    }
}
pub trait ValueBuilder<
    __Context__,
    Value,
>: IsProviderFor<ValueBuilderComponent, __Context__, (Value)> {
    fn build_value(__context__: &__Context__) -> Value;
}
impl<__Provider__, __Context__, Value> ValueBuilder<__Context__, Value> for __Provider__
where
    __Provider__: DelegateComponent<ValueBuilderComponent>
        + IsProviderFor<ValueBuilderComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        ValueBuilderComponent,
    >>::Delegate: ValueBuilder<__Context__, Value>,
{
    fn build_value(__context__: &__Context__) -> Value {
        <__Provider__ as DelegateComponent<
            ValueBuilderComponent,
        >>::Delegate::build_value(__context__)
    }
}
pub struct ValueBuilderComponent;
impl<__Context__, Value> ValueBuilder<__Context__, Value> for UseContext
where
    __Context__: CanBuildValue<Value>,
{
    fn build_value(__context__: &__Context__) -> Value {
        __Context__::build_value(__context__)
    }
}
impl<__Context__, Value> IsProviderFor<ValueBuilderComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanBuildValue<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> ValueBuilder<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: ValueBuilder<__Context__, Value>,
{
    fn build_value(__context__: &__Context__) -> Value {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::build_value(__context__)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<ValueBuilderComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<ValueBuilderComponent, __Context__, (Value)>
        + ValueBuilder<__Context__, Value>,
{}
impl<__Context__> ValueBuilder<__Context__, u64> for BuildU64 {
    fn build_value(__context__: &__Context__) -> u64 {
        0
    }
}
impl<__Context__> IsProviderFor<ValueBuilderComponent, __Context__, (u64)> for BuildU64 {}
pub struct BuildU64;
pub trait HasRecordBuilder {
    type RecordBuilder: Default;
}
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
impl<__Context__, Value, Builder> ValueBuilder<__Context__, Value> for BuildRecord
where
    Value: HasFields + HasRecordBuilder<RecordBuilder = Builder>,
    Value::Fields: BuildFields<__Context__, Builder>,
{
    fn build_value(__context__: &__Context__) -> Value {
        ::core::panicking::panic("not yet implemented")
    }
}
impl<
    __Context__,
    Value,
    Builder,
> IsProviderFor<ValueBuilderComponent, __Context__, (Value)> for BuildRecord
where
    Value: HasFields + HasRecordBuilder<RecordBuilder = Builder>,
    Value::Fields: BuildFields<__Context__, Builder>,
{}
pub struct BuildRecord;
pub struct Inner {
    pub value: u64,
}
impl HasFields for Inner {
    type Fields = Product![Field<Symbol!("value"), u64>];
}
impl HasFieldsRef for Inner {
    type FieldsRef<'__a> = Product![Field<Symbol!("value"), &'__a u64>]
    where
        Self: '__a;
}
impl FromFields for Inner {
    fn from_fields(Cons(value, Nil): Self::Fields) -> Self {
        Self { value: value.value }
    }
}
impl ToFields for Inner {
    fn to_fields(self) -> Self::Fields {
        Cons(self.value.into(), Nil)
    }
}
impl ToFieldsRef for Inner {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.value).into(), Nil)
    }
}
pub struct Outer {
    pub id: u64,
    pub inner: Inner,
}
impl HasFields for Outer {
    type Fields = Product![
        Field<Symbol!("id"), u64>, Field<Symbol!("inner"), Inner>
    ];
}
impl HasFieldsRef for Outer {
    type FieldsRef<'__a> = Product![
        Field<Symbol!("id"), &'__a u64>, Field<Symbol!("inner"), &'__a Inner>
    ]
    where
        Self: '__a;
}
impl FromFields for Outer {
    fn from_fields(Cons(id, Cons(inner, Nil)): Self::Fields) -> Self {
        Self {
            id: id.value,
            inner: inner.value,
        }
    }
}
impl ToFields for Outer {
    fn to_fields(self) -> Self::Fields {
        Cons(self.id.into(), Cons(self.inner.into(), Nil))
    }
}
impl ToFieldsRef for Outer {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.id).into(), Cons((&self.inner).into(), Nil))
    }
}
impl HasRecordBuilder for Inner {
    type RecordBuilder = ();
}
impl HasRecordBuilder for Outer {
    type RecordBuilder = ();
}
pub struct App;
impl DelegateComponent<ValueBuilderComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ValueBuilderComponent)>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ValueBuilderComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@ValueBuilderComponent),
    >: IsProviderFor<ValueBuilderComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueBuilderComponent, PathCons<u64, __Wildcard__>>>
for App {
    type Delegate = BuildU64;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueBuilderComponent, PathCons<u64, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    BuildU64: IsProviderFor<
        PathCons<ValueBuilderComponent, PathCons<u64, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueBuilderComponent, PathCons<Outer, __Wildcard__>>>
for App {
    type Delegate = BuildRecord;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueBuilderComponent, PathCons<Outer, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    BuildRecord: IsProviderFor<
        PathCons<ValueBuilderComponent, PathCons<Outer, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ValueBuilderComponent, Outer> for App {}
fn main() {}
