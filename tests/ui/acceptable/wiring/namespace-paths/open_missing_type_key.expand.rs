#![feature(prelude_import)]
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
//! leaf](../../../../../docs/implementation/typed-root-cause-resolution.md)) rather than reading only
//! its ADT item name (`PathCons`); and the `Path!` resugarer renders the value segment `Vec<u8>`
//! verbatim rather than declining it for carrying generics, so the path folds to
//! `@ItemEncoderComponent.Vec<u8>`.
//!
//! See docs/implementation/typed-root-cause-resolution.md (the missing-redirect-wiring leaf).
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
impl<__Context__, Value> ItemEncoder<__Context__, Value> for EncodeDisplay
where
    Value: core::fmt::Display,
{
    fn encode_item(__context__: &__Context__, value: &Value) -> String {
        value.to_string()
    }
}
impl<__Context__, Value> IsProviderFor<ItemEncoderComponent, __Context__, (Value)>
for EncodeDisplay
where
    Value: core::fmt::Display,
{}
pub struct EncodeDisplay;
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
> DelegateComponent<PathCons<ItemEncoderComponent, PathCons<u64, __Wildcard__>>>
for App {
    type Delegate = EncodeDisplay;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ItemEncoderComponent, PathCons<u64, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    EncodeDisplay: IsProviderFor<
        PathCons<ItemEncoderComponent, PathCons<u64, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ItemEncoderComponent, u64> for App {}
impl __CheckApp<ItemEncoderComponent, Vec<u8>> for App {}
fn main() {}
