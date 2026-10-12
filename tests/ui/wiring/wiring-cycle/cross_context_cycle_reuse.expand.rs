#![feature(prelude_import)]
//! A wiring cycle through two contexts, whose shared node is cached under one walk and must not be
//! reused under another that would cut it. `Inner`'s `DoN` needs a `name` field `Inner` lacks and
//! the plain trait bound `Outer: Linked`; `Linked` holds for `Outer` only through `CanM`, whose
//! provider needs `Inner: CanN` back.
//!
//! The walk from `Inner: CanN` reads `Outer: Linked` as a leaf, since a plain bound on another
//! type is not descended, and caches the complete subtree. The walk from `Outer: CanR` descends
//! `Outer: Linked` (a bound on its own context), reaches `Inner: CanN`, and finds that cached
//! subtree. Its reachable set holds `Outer: Linked`, which is now an ancestor, so the walk resolves
//! the node afresh and the cycle guard cuts the loop.
//!
//! Both errors bottom out on `Inner`'s missing `name` field, so they merge into one block with a
//! caret on one check entry and a label on the other. The merged tree shows both routes and
//! `Inner`'s own `Outer: Linked` leaf, so the merged output does not show whether `Outer`'s walk
//! reused the cached subtree; `cross_context_cycle_reuse_distinct` pins that rule instead.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the reuse rule is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanN {
    fn n(&self);
}
impl<__Context__> CanN for __Context__
where
    __Context__: ProviderN<__Context__>,
{
    fn n(&self) {
        __Context__::n(self)
    }
}
pub trait ProviderN<__Context__>: IsProviderFor<ProviderNComponent, __Context__, ()> {
    fn n(__context__: &__Context__);
}
impl<__Provider__, __Context__> ProviderN<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ProviderNComponent>
        + IsProviderFor<ProviderNComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ProviderNComponent,
    >>::Delegate: ProviderN<__Context__>,
{
    fn n(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<ProviderNComponent>>::Delegate::n(__context__)
    }
}
pub struct ProviderNComponent;
impl<__Context__> ProviderN<__Context__> for UseContext
where
    __Context__: CanN,
{
    fn n(__context__: &__Context__) {
        __Context__::n(__context__)
    }
}
impl<__Context__> IsProviderFor<ProviderNComponent, __Context__, ()> for UseContext
where
    __Context__: CanN,
{}
impl<__Context__, __Components__, __Path__> ProviderN<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: ProviderN<__Context__>,
{
    fn n(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::n(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ProviderNComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ProviderNComponent, __Context__, ()>
        + ProviderN<__Context__>,
{}
pub trait CanM {
    fn m(&self);
}
impl<__Context__> CanM for __Context__
where
    __Context__: ProviderM<__Context__>,
{
    fn m(&self) {
        __Context__::m(self)
    }
}
pub trait ProviderM<__Context__>: IsProviderFor<ProviderMComponent, __Context__, ()> {
    fn m(__context__: &__Context__);
}
impl<__Provider__, __Context__> ProviderM<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ProviderMComponent>
        + IsProviderFor<ProviderMComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ProviderMComponent,
    >>::Delegate: ProviderM<__Context__>,
{
    fn m(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<ProviderMComponent>>::Delegate::m(__context__)
    }
}
pub struct ProviderMComponent;
impl<__Context__> ProviderM<__Context__> for UseContext
where
    __Context__: CanM,
{
    fn m(__context__: &__Context__) {
        __Context__::m(__context__)
    }
}
impl<__Context__> IsProviderFor<ProviderMComponent, __Context__, ()> for UseContext
where
    __Context__: CanM,
{}
impl<__Context__, __Components__, __Path__> ProviderM<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: ProviderM<__Context__>,
{
    fn m(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::m(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ProviderMComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ProviderMComponent, __Context__, ()>
        + ProviderM<__Context__>,
{}
pub trait CanR {
    fn r(&self);
}
impl<__Context__> CanR for __Context__
where
    __Context__: ProviderR<__Context__>,
{
    fn r(&self) {
        __Context__::r(self)
    }
}
pub trait ProviderR<__Context__>: IsProviderFor<ProviderRComponent, __Context__, ()> {
    fn r(__context__: &__Context__);
}
impl<__Provider__, __Context__> ProviderR<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ProviderRComponent>
        + IsProviderFor<ProviderRComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ProviderRComponent,
    >>::Delegate: ProviderR<__Context__>,
{
    fn r(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<ProviderRComponent>>::Delegate::r(__context__)
    }
}
pub struct ProviderRComponent;
impl<__Context__> ProviderR<__Context__> for UseContext
where
    __Context__: CanR,
{
    fn r(__context__: &__Context__) {
        __Context__::r(__context__)
    }
}
impl<__Context__> IsProviderFor<ProviderRComponent, __Context__, ()> for UseContext
where
    __Context__: CanR,
{}
impl<__Context__, __Components__, __Path__> ProviderR<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: ProviderR<__Context__>,
{
    fn r(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::r(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ProviderRComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ProviderRComponent, __Context__, ()>
        + ProviderR<__Context__>,
{}
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
pub trait Linked {}
impl<Context> Linked for Context
where
    Context: CanM,
{}
pub struct Inner {}
pub struct Outer {}
impl<__Context__, Partner> ProviderN<__Context__> for DoN<Partner>
where
    Partner: Linked,
    __Context__: HasName,
{
    fn n(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__, Partner> IsProviderFor<ProviderNComponent, __Context__, ()>
for DoN<Partner>
where
    Partner: Linked,
    __Context__: HasName,
{}
pub struct DoN<Partner>(pub ::core::marker::PhantomData<Partner>);
impl<__Context__, Partner> ProviderM<__Context__> for DoM<Partner>
where
    Partner: CanN,
{
    fn m(__context__: &__Context__) {}
}
impl<__Context__, Partner> IsProviderFor<ProviderMComponent, __Context__, ()>
for DoM<Partner>
where
    Partner: CanN,
{}
pub struct DoM<Partner>(pub ::core::marker::PhantomData<Partner>);
impl<__Context__> ProviderR<__Context__> for DoR
where
    __Context__: Linked,
{
    fn r(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<ProviderRComponent, __Context__, ()> for DoR
where
    __Context__: Linked,
{}
pub struct DoR;
impl DelegateComponent<ProviderNComponent> for Inner {
    type Delegate = DoN<Outer>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ProviderNComponent, __Context__, __Params__> for Inner
where
    DoN<Outer>: IsProviderFor<ProviderNComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ProviderMComponent> for Outer {
    type Delegate = DoM<Inner>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ProviderMComponent, __Context__, __Params__> for Outer
where
    DoM<Inner>: IsProviderFor<ProviderMComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ProviderRComponent> for Outer {
    type Delegate = DoR;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ProviderRComponent, __Context__, __Params__> for Outer
where
    DoR: IsProviderFor<ProviderRComponent, __Context__, __Params__>,
{}
trait __CheckInner<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckInner<ProviderNComponent, ()> for Inner {}
trait __CheckOuter<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckOuter<ProviderRComponent, ()> for Outer {}
fn main() {}
