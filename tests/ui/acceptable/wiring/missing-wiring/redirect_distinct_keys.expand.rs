#![feature(prelude_import)]
//! Acceptable: two dependencies dispatched along the same `open` route for distinct, unwired value
//! types — the shape that pins two redirect hops staying distinct nodes across branches.
//!
//! `AssembleParts` (wired directly to `AssemblerComponent`, *not* through a redirect) needs to build
//! two values, a `Left` and a `Right`, through the `open`-dispatched `ValueBuilder`. Neither type is
//! wired, so checking `CanAssemble` fails with two root causes: the missing
//! `@ValueBuilderComponent.Left` and `@ValueBuilderComponent.Right` wirings. Each is reached through a
//! `redirect lookup to @ValueBuilderComponent` hop — the same route, rendered identically — but for a
//! different dispatch key.
//!
//! Because the top component is wired directly, each branch's redirect is the *first* redirect on its
//! path, so the two are compared for identity against each other. They render the same label yet are
//! different lookups; the dependency graph keys a redirect node's identity on the dispatched key as
//! well as the route, so the two stay distinct: the tree branches to each value's own redirect and
//! its own missing-wiring leaf, rather than collapsing both leaves under one shared redirect with the
//! other branch reduced to a `(*)` back-reference.
//!
//! See docs/implementation/dependency-graph-rendering.md (node identity is cross-path, keyed on the
//! dispatched value).
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
pub struct Left;
pub struct Right;
pub trait CanAssemble {
    fn assemble(&self);
}
impl<__Context__> CanAssemble for __Context__
where
    __Context__: Assembler<__Context__>,
{
    fn assemble(&self) {
        __Context__::assemble(self)
    }
}
pub trait Assembler<__Context__>: IsProviderFor<AssemblerComponent, __Context__, ()> {
    fn assemble(__context__: &__Context__);
}
impl<__Provider__, __Context__> Assembler<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<AssemblerComponent>
        + IsProviderFor<AssemblerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        AssemblerComponent,
    >>::Delegate: Assembler<__Context__>,
{
    fn assemble(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            AssemblerComponent,
        >>::Delegate::assemble(__context__)
    }
}
pub struct AssemblerComponent;
impl<__Context__> Assembler<__Context__> for UseContext
where
    __Context__: CanAssemble,
{
    fn assemble(__context__: &__Context__) {
        __Context__::assemble(__context__)
    }
}
impl<__Context__> IsProviderFor<AssemblerComponent, __Context__, ()> for UseContext
where
    __Context__: CanAssemble,
{}
impl<__Context__, __Components__, __Path__> Assembler<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Assembler<__Context__>,
{
    fn assemble(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::assemble(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<AssemblerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<AssemblerComponent, __Context__, ()>
        + Assembler<__Context__>,
{}
impl<__Context__> Assembler<__Context__> for AssembleParts
where
    __Context__: CanBuildValue<Left> + CanBuildValue<Right>,
{
    fn assemble(__context__: &__Context__) {
        let _: Left = __context__.build_value();
        let _: Right = __context__.build_value();
    }
}
impl<__Context__> IsProviderFor<AssemblerComponent, __Context__, ()> for AssembleParts
where
    __Context__: CanBuildValue<Left> + CanBuildValue<Right>,
{}
pub struct AssembleParts;
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
impl DelegateComponent<AssemblerComponent> for App {
    type Delegate = AssembleParts;
}
impl<__Context__, __Params__> IsProviderFor<AssemblerComponent, __Context__, __Params__>
for App
where
    AssembleParts: IsProviderFor<AssemblerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<AssemblerComponent, ()> for App {}
fn main() {}
