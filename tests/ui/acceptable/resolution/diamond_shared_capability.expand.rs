#![feature(prelude_import)]
//! Diamond reuse in the resolver's walk: one shared capability reached from two independent
//! branches of a single dependency tree. `CanTop` depends on both `CanLeft` and `CanRight`, and
//! each of those depends on the same `CanShared` capability, whose provider needs the `name` field.
//! `App` wires all four components but has no `name` field, so the walk from `CanTop` descends into
//! `App: CanShared` twice — once under `CanLeft`, once under `CanRight` — the diamond the
//! per-node [resolution cache](../../../../docs/implementation/cached-dependency-resolution.md)
//! resolves once and reuses.
//!
//! Because both branches bottom out on the *same* missing field, they are one root cause with two
//! paths, which the [dependency graph](../../../../docs/implementation/dependency-graph-rendering.md)
//! renders as a diamond: `CanTop` branches to `CanLeft` and `CanRight`, the shared `CanShared`
//! subtree is drawn in full under the first (`CanLeft`) and referenced with `(*)` under the second
//! (`CanRight`), and the missing `name` field is shown once. The point of the fixture is that both
//! branches appear — neither is dropped — while the shared subtree is not duplicated. It also pins
//! the cache: `CanShared` renders identically whichever branch reaches it first, so a cache hit on
//! the second branch is output-preserving.
//!
//! See docs/implementation/dependency-graph-rendering.md (diamond) and
//! docs/implementation/cached-dependency-resolution.md (diamond reuse).
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
pub trait CanShared {
    fn shared(&self);
}
impl<__Context__> CanShared for __Context__
where
    __Context__: SharedProvider<__Context__>,
{
    fn shared(&self) {
        __Context__::shared(self)
    }
}
pub trait SharedProvider<
    __Context__,
>: IsProviderFor<SharedProviderComponent, __Context__, ()> {
    fn shared(__context__: &__Context__);
}
impl<__Provider__, __Context__> SharedProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<SharedProviderComponent>
        + IsProviderFor<SharedProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        SharedProviderComponent,
    >>::Delegate: SharedProvider<__Context__>,
{
    fn shared(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            SharedProviderComponent,
        >>::Delegate::shared(__context__)
    }
}
pub struct SharedProviderComponent;
impl<__Context__> SharedProvider<__Context__> for UseContext
where
    __Context__: CanShared,
{
    fn shared(__context__: &__Context__) {
        __Context__::shared(__context__)
    }
}
impl<__Context__> IsProviderFor<SharedProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanShared,
{}
impl<__Context__, __Components__, __Path__> SharedProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: SharedProvider<__Context__>,
{
    fn shared(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::shared(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<SharedProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<SharedProviderComponent, __Context__, ()>
        + SharedProvider<__Context__>,
{}
pub trait CanLeft {
    fn left(&self);
}
impl<__Context__> CanLeft for __Context__
where
    __Context__: LeftProvider<__Context__>,
{
    fn left(&self) {
        __Context__::left(self)
    }
}
pub trait LeftProvider<
    __Context__,
>: IsProviderFor<LeftProviderComponent, __Context__, ()> {
    fn left(__context__: &__Context__);
}
impl<__Provider__, __Context__> LeftProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<LeftProviderComponent>
        + IsProviderFor<LeftProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        LeftProviderComponent,
    >>::Delegate: LeftProvider<__Context__>,
{
    fn left(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            LeftProviderComponent,
        >>::Delegate::left(__context__)
    }
}
pub struct LeftProviderComponent;
impl<__Context__> LeftProvider<__Context__> for UseContext
where
    __Context__: CanLeft,
{
    fn left(__context__: &__Context__) {
        __Context__::left(__context__)
    }
}
impl<__Context__> IsProviderFor<LeftProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanLeft,
{}
impl<__Context__, __Components__, __Path__> LeftProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: LeftProvider<__Context__>,
{
    fn left(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::left(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<LeftProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<LeftProviderComponent, __Context__, ()>
        + LeftProvider<__Context__>,
{}
pub trait CanRight {
    fn right(&self);
}
impl<__Context__> CanRight for __Context__
where
    __Context__: RightProvider<__Context__>,
{
    fn right(&self) {
        __Context__::right(self)
    }
}
pub trait RightProvider<
    __Context__,
>: IsProviderFor<RightProviderComponent, __Context__, ()> {
    fn right(__context__: &__Context__);
}
impl<__Provider__, __Context__> RightProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<RightProviderComponent>
        + IsProviderFor<RightProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        RightProviderComponent,
    >>::Delegate: RightProvider<__Context__>,
{
    fn right(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            RightProviderComponent,
        >>::Delegate::right(__context__)
    }
}
pub struct RightProviderComponent;
impl<__Context__> RightProvider<__Context__> for UseContext
where
    __Context__: CanRight,
{
    fn right(__context__: &__Context__) {
        __Context__::right(__context__)
    }
}
impl<__Context__> IsProviderFor<RightProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanRight,
{}
impl<__Context__, __Components__, __Path__> RightProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: RightProvider<__Context__>,
{
    fn right(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::right(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<RightProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<RightProviderComponent, __Context__, ()>
        + RightProvider<__Context__>,
{}
pub trait CanTop {
    fn top(&self);
}
impl<__Context__> CanTop for __Context__
where
    __Context__: TopProvider<__Context__>,
{
    fn top(&self) {
        __Context__::top(self)
    }
}
pub trait TopProvider<
    __Context__,
>: IsProviderFor<TopProviderComponent, __Context__, ()> {
    fn top(__context__: &__Context__);
}
impl<__Provider__, __Context__> TopProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<TopProviderComponent>
        + IsProviderFor<TopProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TopProviderComponent,
    >>::Delegate: TopProvider<__Context__>,
{
    fn top(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            TopProviderComponent,
        >>::Delegate::top(__context__)
    }
}
pub struct TopProviderComponent;
impl<__Context__> TopProvider<__Context__> for UseContext
where
    __Context__: CanTop,
{
    fn top(__context__: &__Context__) {
        __Context__::top(__context__)
    }
}
impl<__Context__> IsProviderFor<TopProviderComponent, __Context__, ()> for UseContext
where
    __Context__: CanTop,
{}
impl<__Context__, __Components__, __Path__> TopProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: TopProvider<__Context__>,
{
    fn top(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::top(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TopProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TopProviderComponent, __Context__, ()>
        + TopProvider<__Context__>,
{}
impl<__Context__> SharedProvider<__Context__> for ProvideShared
where
    __Context__: HasName,
{
    fn shared(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__> IsProviderFor<SharedProviderComponent, __Context__, ()>
for ProvideShared
where
    __Context__: HasName,
{}
pub struct ProvideShared;
impl<__Context__> LeftProvider<__Context__> for ProvideLeft
where
    __Context__: CanShared,
{
    fn left(__context__: &__Context__) {
        __context__.shared();
    }
}
impl<__Context__> IsProviderFor<LeftProviderComponent, __Context__, ()> for ProvideLeft
where
    __Context__: CanShared,
{}
pub struct ProvideLeft;
impl<__Context__> RightProvider<__Context__> for ProvideRight
where
    __Context__: CanShared,
{
    fn right(__context__: &__Context__) {
        __context__.shared();
    }
}
impl<__Context__> IsProviderFor<RightProviderComponent, __Context__, ()> for ProvideRight
where
    __Context__: CanShared,
{}
pub struct ProvideRight;
impl<__Context__> TopProvider<__Context__> for ProvideTop
where
    __Context__: CanLeft + CanRight,
{
    fn top(__context__: &__Context__) {
        __context__.left();
        __context__.right();
    }
}
impl<__Context__> IsProviderFor<TopProviderComponent, __Context__, ()> for ProvideTop
where
    __Context__: CanLeft + CanRight,
{}
pub struct ProvideTop;
pub struct App {
    pub age: u8,
}
impl HasField<Symbol!("age")> for App {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &Self::Value {
        &self.age
    }
}
impl HasFieldMut<Symbol!("age")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &mut Self::Value {
        &mut self.age
    }
}
impl DelegateComponent<SharedProviderComponent> for App {
    type Delegate = ProvideShared;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<SharedProviderComponent, __Context__, __Params__> for App
where
    ProvideShared: IsProviderFor<SharedProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<LeftProviderComponent> for App {
    type Delegate = ProvideLeft;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<LeftProviderComponent, __Context__, __Params__> for App
where
    ProvideLeft: IsProviderFor<LeftProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<RightProviderComponent> for App {
    type Delegate = ProvideRight;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<RightProviderComponent, __Context__, __Params__> for App
where
    ProvideRight: IsProviderFor<RightProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TopProviderComponent> for App {
    type Delegate = ProvideTop;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<TopProviderComponent, __Context__, __Params__> for App
where
    ProvideTop: IsProviderFor<TopProviderComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<TopProviderComponent, ()> for App {}
fn main() {}
