#![feature(prelude_import)]
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
impl<__Context__, Item> ItemEncoder<__Context__, Seq<Item>> for EncodeSeq
where
    __Context__: CanEncodeItem<Item>,
{
    fn encode_item(__context__: &__Context__, value: &Seq<Item>) -> String {
        let mut out = String::new();
        for item in value.0.iter() {
            out.push_str(&__context__.encode_item(item));
        }
        out
    }
}
impl<__Context__, Item> IsProviderFor<ItemEncoderComponent, __Context__, (Seq<Item>)>
for EncodeSeq
where
    __Context__: CanEncodeItem<Item>,
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
fn main() {
    let app = App;
    let _ = app
        .encode_item(
            &Seq(
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                    ::alloc::intrinsics::write_box_via_move(
                        ::alloc::boxed::Box::new_uninit(),
                        [1u64],
                    ),
                ),
            ),
        );
}
