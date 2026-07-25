#![feature(prelude_import)]
//! A wiring failure threading the record field-list machinery through *nested*
//! higher-ranked (`for<'a>`) hops, resolved to its root cause.
//!
//! This is the faithful shape of `cgp-serde`'s `MessagesArchive` check. Encoding a record
//! walks its `HasFields` list (`EncodeFields` over `Cons`/`Field`/`Nil`), each field
//! encoded through the context; a `Vec` field routes through `EncodeIterator`, whose
//! higher-ranked dependency `Self: for<'a> CanEncodeValue<<&'a Value as IntoIterator>::Item>`
//! carries a projection through the bound lifetime; and the borrowed item routes back to
//! the owned value through `EncodeDeref`. `Outer` nests this twice — `Outer` → `Vec<Inner>`
//! → `Inner` → `Vec<u64>` → `u64` — and `u64` is never wired.
//!
//! The single-hop `higher_ranked_descent` and the record-only `record_field_chain` pin the
//! two mechanics separately; this fixture pins their combination — the record-list
//! recursion crossing two nested higher-ranked projection hops — descending all the way to
//! the one missing `@ValueEncoderComponent.u64` wiring, each nesting level a distinct hop
//! in the chain.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanEncodeValue<Value: ?Sized> {
    fn encode_value(&self, value: &Value) -> String;
}
impl<__Context__, Value: ?Sized> CanEncodeValue<Value> for __Context__
where
    __Context__: ValueEncoder<__Context__, Value>,
{
    fn encode_value(&self, value: &Value) -> String {
        __Context__::encode_value(self, value)
    }
}
pub trait ValueEncoder<
    __Context__,
    Value: ?Sized,
>: IsProviderFor<ValueEncoderComponent, __Context__, (Value)> {
    fn encode_value(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value: ?Sized> ValueEncoder<__Context__, Value>
for __Provider__
where
    __Provider__: DelegateComponent<ValueEncoderComponent>
        + IsProviderFor<ValueEncoderComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        ValueEncoderComponent,
    >>::Delegate: ValueEncoder<__Context__, Value>,
{
    fn encode_value(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            ValueEncoderComponent,
        >>::Delegate::encode_value(__context__, value)
    }
}
pub struct ValueEncoderComponent;
impl<__Context__, Value: ?Sized> ValueEncoder<__Context__, Value> for UseContext
where
    __Context__: CanEncodeValue<Value>,
{
    fn encode_value(__context__: &__Context__, value: &Value) -> String {
        __Context__::encode_value(__context__, value)
    }
}
impl<
    __Context__,
    Value: ?Sized,
> IsProviderFor<ValueEncoderComponent, __Context__, (Value)> for UseContext
where
    __Context__: CanEncodeValue<Value>,
{}
impl<
    __Context__,
    Value: ?Sized,
    __Components__,
    __Path__,
> ValueEncoder<__Context__, Value> for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: ValueEncoder<__Context__, Value>,
{
    fn encode_value(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::encode_value(__context__, value)
    }
}
impl<
    __Context__,
    Value: ?Sized,
    __Components__,
    __Path__,
> IsProviderFor<ValueEncoderComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<ValueEncoderComponent, __Context__, (Value)>
        + ValueEncoder<__Context__, Value>,
{}
impl<__Context__> ValueEncoder<__Context__, u64> for EncodeU64 {
    fn encode_value(__context__: &__Context__, value: &u64) -> String {
        value.to_string()
    }
}
impl<__Context__> IsProviderFor<ValueEncoderComponent, __Context__, (u64)>
for EncodeU64 {}
pub struct EncodeU64;
impl<__Context__, Value> ValueEncoder<__Context__, Value> for EncodeIterator
where
    for<'a> &'a Value: IntoIterator,
    __Context__: for<'a> CanEncodeValue<<&'a Value as IntoIterator>::Item>,
{
    fn encode_value(__context__: &__Context__, _value: &Value) -> String {
        String::new()
    }
}
impl<__Context__, Value> IsProviderFor<ValueEncoderComponent, __Context__, (Value)>
for EncodeIterator
where
    for<'a> &'a Value: IntoIterator,
    __Context__: for<'a> CanEncodeValue<<&'a Value as IntoIterator>::Item>,
{}
pub struct EncodeIterator;
impl<'a, __Context__, Value> ValueEncoder<__Context__, &'a Value> for EncodeDeref
where
    __Context__: CanEncodeValue<Value>,
{
    fn encode_value(__context__: &__Context__, value: &&'a Value) -> String {
        __context__.encode_value(*value)
    }
}
impl<
    'a,
    __Context__,
    Value,
> IsProviderFor<ValueEncoderComponent, __Context__, (&'a Value)> for EncodeDeref
where
    __Context__: CanEncodeValue<Value>,
{}
pub struct EncodeDeref;
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
impl<__Context__, Value> ValueEncoder<__Context__, Value> for EncodeRecord
where
    Value: HasFields,
    Value::Fields: EncodeFields<__Context__, Value>,
{
    fn encode_value(__context__: &__Context__, _value: &Value) -> String {
        String::new()
    }
}
impl<__Context__, Value> IsProviderFor<ValueEncoderComponent, __Context__, (Value)>
for EncodeRecord
where
    Value: HasFields,
    Value::Fields: EncodeFields<__Context__, Value>,
{}
pub struct EncodeRecord;
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
pub struct Mid {
    pub inners: Vec<Inner>,
}
impl HasFields for Mid {
    type Fields = Product![Field<Symbol!("inners"), Vec<Inner>>];
}
impl HasFieldsRef for Mid {
    type FieldsRef<'__a> = Product![Field<Symbol!("inners"), &'__a Vec<Inner>>]
    where
        Self: '__a;
}
impl FromFields for Mid {
    fn from_fields(Cons(inners, Nil): Self::Fields) -> Self {
        Self { inners: inners.value }
    }
}
impl ToFields for Mid {
    fn to_fields(self) -> Self::Fields {
        Cons(self.inners.into(), Nil)
    }
}
impl ToFieldsRef for Mid {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.inners).into(), Nil)
    }
}
pub struct Outer {
    pub mids: Vec<Mid>,
}
impl HasFields for Outer {
    type Fields = Product![Field<Symbol!("mids"), Vec<Mid>>];
}
impl HasFieldsRef for Outer {
    type FieldsRef<'__a> = Product![Field<Symbol!("mids"), &'__a Vec<Mid>>]
    where
        Self: '__a;
}
impl FromFields for Outer {
    fn from_fields(Cons(mids, Nil): Self::Fields) -> Self {
        Self { mids: mids.value }
    }
}
impl ToFields for Outer {
    fn to_fields(self) -> Self::Fields {
        Cons(self.mids.into(), Nil)
    }
}
impl ToFieldsRef for Outer {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        Cons((&self.mids).into(), Nil)
    }
}
pub struct App;
impl DelegateComponent<ValueEncoderComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ValueEncoderComponent)>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ValueEncoderComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@ValueEncoderComponent),
    >: IsProviderFor<ValueEncoderComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueEncoderComponent, PathCons<Outer, __Wildcard__>>>
for App {
    type Delegate = EncodeRecord;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueEncoderComponent, PathCons<Outer, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeRecord: IsProviderFor<
        PathCons<ValueEncoderComponent, PathCons<Outer, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueEncoderComponent, PathCons<Mid, __Wildcard__>>>
for App {
    type Delegate = EncodeRecord;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueEncoderComponent, PathCons<Mid, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeRecord: IsProviderFor<
        PathCons<ValueEncoderComponent, PathCons<Mid, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueEncoderComponent, PathCons<Inner, __Wildcard__>>>
for App {
    type Delegate = EncodeRecord;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueEncoderComponent, PathCons<Inner, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeRecord: IsProviderFor<
        PathCons<ValueEncoderComponent, PathCons<Inner, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueEncoderComponent, PathCons<Vec<Mid>, __Wildcard__>>>
for App {
    type Delegate = EncodeIterator;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueEncoderComponent, PathCons<Vec<Mid>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeIterator: IsProviderFor<
        PathCons<ValueEncoderComponent, PathCons<Vec<Mid>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ValueEncoderComponent, PathCons<Vec<Inner>, __Wildcard__>>>
for App {
    type Delegate = EncodeIterator;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueEncoderComponent, PathCons<Vec<Inner>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeIterator: IsProviderFor<
        PathCons<ValueEncoderComponent, PathCons<Vec<Inner>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    'a,
    T,
    __Wildcard__,
> DelegateComponent<PathCons<ValueEncoderComponent, PathCons<&'a T, __Wildcard__>>>
for App {
    type Delegate = EncodeDeref;
}
impl<
    'a,
    T,
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ValueEncoderComponent, PathCons<&'a T, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeDeref: IsProviderFor<
        PathCons<ValueEncoderComponent, PathCons<&'a T, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ValueEncoderComponent, Outer> for App {}
fn main() {}
