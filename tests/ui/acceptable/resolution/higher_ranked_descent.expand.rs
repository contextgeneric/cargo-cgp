#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanEncodeItem<Value> {
    fn encode_item(&self, value: &Value) -> String;
}
impl<__Context__, Value> CanEncodeItem<Value> for __Context__
where
    __Context__: ItemEncoder<__Context__, Value>,
{
    fn encode_item(&self, value: &Value) -> String {
        __Context__::encode_item(self, value)
    }
}
pub trait ItemEncoder<
    __Context__,
    Value,
>: IsProviderFor<ItemEncoderComponent, __Context__, (Value)> {
    fn encode_item(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value> ItemEncoder<__Context__, Value> for __Provider__
where
    __Provider__: DelegateComponent<ItemEncoderComponent>
        + IsProviderFor<ItemEncoderComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        ItemEncoderComponent,
    >>::Delegate: ItemEncoder<__Context__, Value>,
{
    fn encode_item(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            ItemEncoderComponent,
        >>::Delegate::encode_item(__context__, value)
    }
}
pub struct ItemEncoderComponent;
impl<__Context__, Value> ItemEncoder<__Context__, Value> for UseContext
where
    __Context__: CanEncodeItem<Value>,
{
    fn encode_item(__context__: &__Context__, value: &Value) -> String {
        __Context__::encode_item(__context__, value)
    }
}
impl<__Context__, Value> IsProviderFor<ItemEncoderComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanEncodeItem<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> ItemEncoder<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: ItemEncoder<__Context__, Value>,
{
    fn encode_item(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::encode_item(__context__, value)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<ItemEncoderComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<ItemEncoderComponent, __Context__, (Value)>
        + ItemEncoder<__Context__, Value>,
{}
pub struct Seq<T>(pub Vec<T>);
impl<__Context__, Value> ItemEncoder<__Context__, Seq<Value>> for EncodeSeq
where
    __Context__: for<'a> CanEncodeItem<&'a Value>,
{
    fn encode_item(__context__: &__Context__, _value: &Seq<Value>) -> String {
        String::new()
    }
}
impl<__Context__, Value> IsProviderFor<ItemEncoderComponent, __Context__, (Seq<Value>)>
for EncodeSeq
where
    __Context__: for<'a> CanEncodeItem<&'a Value>,
{}
pub struct EncodeSeq;
pub struct App;
impl DelegateComponent<ItemEncoderComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ItemEncoderComponent)>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ItemEncoderComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@ItemEncoderComponent),
    >: IsProviderFor<ItemEncoderComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ItemEncoderComponent, PathCons<Seq<u64>, __Wildcard__>>>
for App {
    type Delegate = EncodeSeq;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ItemEncoderComponent, PathCons<Seq<u64>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeSeq: IsProviderFor<
        PathCons<ItemEncoderComponent, PathCons<Seq<u64>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ItemEncoderComponent, Seq<u64>> for App {}
fn main() {}
