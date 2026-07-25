#![feature(prelude_import)]
//! Two genuinely distinct root causes that happen to *name* the same thing, kept apart. `Outer`'s
//! provider reads a `name` field from its own context and also depends on the concrete `Inner`
//! context being able to compute, whose provider reads a `name` field of *its* own. Neither struct
//! carries one, so `Outer`'s dependency tree bottoms out on two missing fields that share a name but
//! sit on different structs — and need two separate fixes.
//!
//! Causes are grouped by whole-leaf equality, so the two stay distinct and the note heads a
//! `root causes:` list naming both owners. Grouping them by the field name alone would merge them
//! into one cause whose singular heading named only the first struct, while the tree below it still
//! branched to both — a heading that understates the work.
//!
//! The cross-context machinery this leans on (the `where Inner: CanCompute` recovery, the re-rooting
//! that makes `Inner`'s subtree decode against `Inner`) is pinned separately by
//! [`cross_context_node_key`](cross_context_node_key.rs); here it is only the shortest way to reach
//! two same-named fields on different owners from one check entry.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
pub trait CanCompute {
    fn compute(&self);
}
impl<__Context__> CanCompute for __Context__
where
    __Context__: Computer<__Context__>,
{
    fn compute(&self) {
        __Context__::compute(self)
    }
}
pub trait Computer<__Context__>: IsProviderFor<ComputerComponent, __Context__, ()> {
    fn compute(__context__: &__Context__);
}
impl<__Provider__, __Context__> Computer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ComputerComponent>
        + IsProviderFor<ComputerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ComputerComponent,
    >>::Delegate: Computer<__Context__>,
{
    fn compute(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            ComputerComponent,
        >>::Delegate::compute(__context__)
    }
}
pub struct ComputerComponent;
impl<__Context__> Computer<__Context__> for UseContext
where
    __Context__: CanCompute,
{
    fn compute(__context__: &__Context__) {
        __Context__::compute(__context__)
    }
}
impl<__Context__> IsProviderFor<ComputerComponent, __Context__, ()> for UseContext
where
    __Context__: CanCompute,
{}
impl<__Context__, __Components__, __Path__> Computer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Computer<__Context__>,
{
    fn compute(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::compute(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ComputerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ComputerComponent, __Context__, ()>
        + Computer<__Context__>,
{}
impl<__Context__> Computer<__Context__> for DoCompute
where
    __Context__: HasName,
{
    fn compute(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__> IsProviderFor<ComputerComponent, __Context__, ()> for DoCompute
where
    __Context__: HasName,
{}
pub struct DoCompute;
pub trait CanRun {
    fn run(&self);
}
impl<__Context__> CanRun for __Context__
where
    __Context__: Runner<__Context__>,
{
    fn run(&self) {
        __Context__::run(self)
    }
}
pub trait Runner<__Context__>: IsProviderFor<RunnerComponent, __Context__, ()> {
    fn run(__context__: &__Context__);
}
impl<__Provider__, __Context__> Runner<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<RunnerComponent>
        + IsProviderFor<RunnerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<RunnerComponent>>::Delegate: Runner<__Context__>,
{
    fn run(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<RunnerComponent>>::Delegate::run(__context__)
    }
}
pub struct RunnerComponent;
impl<__Context__> Runner<__Context__> for UseContext
where
    __Context__: CanRun,
{
    fn run(__context__: &__Context__) {
        __Context__::run(__context__)
    }
}
impl<__Context__> IsProviderFor<RunnerComponent, __Context__, ()> for UseContext
where
    __Context__: CanRun,
{}
impl<__Context__, __Components__, __Path__> Runner<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Runner<__Context__>,
{
    fn run(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::run(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<RunnerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<RunnerComponent, __Context__, ()> + Runner<__Context__>,
{}
impl<__Context__> Runner<__Context__> for RunViaInner
where
    Inner: CanCompute,
    __Context__: HasName,
{
    fn run(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__> IsProviderFor<RunnerComponent, __Context__, ()> for RunViaInner
where
    Inner: CanCompute,
    __Context__: HasName,
{}
pub struct RunViaInner;
pub struct Inner {
    pub age: u8,
}
impl HasField<Symbol!("age")> for Inner {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &Self::Value {
        &self.age
    }
}
impl HasFieldMut<Symbol!("age")> for Inner {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &mut Self::Value {
        &mut self.age
    }
}
pub struct Outer {
    pub label: u8,
}
impl HasField<Symbol!("label")> for Outer {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &Self::Value {
        &self.label
    }
}
impl HasFieldMut<Symbol!("label")> for Outer {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &mut Self::Value {
        &mut self.label
    }
}
impl DelegateComponent<ComputerComponent> for Inner {
    type Delegate = DoCompute;
}
impl<__Context__, __Params__> IsProviderFor<ComputerComponent, __Context__, __Params__>
for Inner
where
    DoCompute: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<RunnerComponent> for Outer {
    type Delegate = RunViaInner;
}
impl<__Context__, __Params__> IsProviderFor<RunnerComponent, __Context__, __Params__>
for Outer
where
    RunViaInner: IsProviderFor<RunnerComponent, __Context__, __Params__>,
{}
trait __CheckOuter<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckOuter<RunnerComponent, ()> for Outer {}
fn main() {}
