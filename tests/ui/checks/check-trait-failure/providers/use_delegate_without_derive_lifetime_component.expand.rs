#![feature(prelude_import)]
//! `UseDelegate` wired for a component with a lifetime parameter that lacks `#[derive_delegate]`.
//!
//! `[CGP-E115]` says `UseDelegate` is not a provider of the component, and its `help` names the
//! `open` statement with one path segment per type parameter of the component. The provider trait
//! takes the lifetime ahead of the context, so the help skips the context by type position and
//! names `Item` alone, never the generated `__Context__`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanLabel<'a, Item: 'a> {
    fn label(&self, item: &'a Item) -> &'a str;
}
impl<'a, __Context__, Item: 'a> CanLabel<'a, Item> for __Context__
where
    __Context__: Labeller<'a, __Context__, Item>,
{
    fn label(&self, item: &'a Item) -> &'a str {
        __Context__::label(self, item)
    }
}
pub trait Labeller<
    'a,
    __Context__,
    Item: 'a,
>: IsProviderFor<LabellerComponent, __Context__, (Life<'a>, Item)> {
    fn label(__context__: &__Context__, item: &'a Item) -> &'a str;
}
impl<'a, __Provider__, __Context__, Item: 'a> Labeller<'a, __Context__, Item>
for __Provider__
where
    __Provider__: DelegateComponent<LabellerComponent>
        + IsProviderFor<LabellerComponent, __Context__, (Life<'a>, Item)>,
    <__Provider__ as DelegateComponent<
        LabellerComponent,
    >>::Delegate: Labeller<'a, __Context__, Item>,
{
    fn label(__context__: &__Context__, item: &'a Item) -> &'a str {
        <__Provider__ as DelegateComponent<
            LabellerComponent,
        >>::Delegate::label(__context__, item)
    }
}
///Component marker for [`CanLabel`].
pub struct LabellerComponent;
impl<'a, __Context__, Item: 'a> Labeller<'a, __Context__, Item> for UseContext
where
    __Context__: CanLabel<'a, Item>,
{
    fn label(__context__: &__Context__, item: &'a Item) -> &'a str {
        __Context__::label(__context__, item)
    }
}
impl<
    'a,
    __Context__,
    Item: 'a,
> IsProviderFor<LabellerComponent, __Context__, (Life<'a>, Item)> for UseContext
where
    __Context__: CanLabel<'a, Item>,
{}
impl<'a, __Context__, Item: 'a, __Components__, __Path__> Labeller<'a, __Context__, Item>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Item)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Item)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Item)>>::Output,
    >>::Delegate: Labeller<'a, __Context__, Item>,
{
    fn label(__context__: &__Context__, item: &'a Item) -> &'a str {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Item)>>::Output,
        >>::Delegate::label(__context__, item)
    }
}
impl<
    'a,
    __Context__,
    Item: 'a,
    __Components__,
    __Path__,
> IsProviderFor<LabellerComponent, __Context__, (Life<'a>, Item)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Item)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Item)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Item)>>::Output,
    >>::Delegate: Labeller<'a, __Context__, Item>,
{}
pub struct Square(pub String);
impl<'a, __Context__> Labeller<'a, __Context__, Square> for SquareLabel {
    fn label(__context__: &__Context__, item: &'a Square) -> &'a str {
        &item.0
    }
}
impl<'a, __Context__> IsProviderFor<LabellerComponent, __Context__, (Life<'a>, Square)>
for SquareLabel {}
///Provider `SquareLabel` for [`Labeller`].
pub struct SquareLabel;
pub struct App;
///Wiring table `LabelTable`.
pub struct LabelTable;
impl DelegateComponent<LabellerComponent> for App {
    type Delegate = UseDelegate<LabelTable>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<LabellerComponent, __Context__, __Params__> for App
where
    UseDelegate<LabelTable>: IsProviderFor<LabellerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<Square> for LabelTable {
    type Delegate = SquareLabel;
}
impl<__Context__, __Params__: ?Sized> IsProviderFor<Square, __Context__, __Params__>
for LabelTable
where
    SquareLabel: IsProviderFor<Square, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl<'a> __CheckApp<LabellerComponent, (Life<'a>, Square)> for App {}
fn main() {}
