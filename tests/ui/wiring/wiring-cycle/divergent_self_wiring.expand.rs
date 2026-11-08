#![feature(prelude_import)]
//! A wiring whose lookup grows in its `Self` type: `ShowWrapped` implements `Shower` for a context
//! by requiring `CanShow` of `Vec<Self>`, and both `u32` and every `Vec<T>` are wired to it, so
//! checking `u32` asks for `Vec<u32>: CanShow`, then `Vec<Vec<u32>>: CanShow`, and so on, until the
//! trait solver overflows (`E0275`).
//!
//! The tool's walk re-roots at each `Vec<…>`, a context since it carries wiring of its own, so it
//! descends the growing requirements until its depth cap cuts the lookup. The overflow reads as
//! `[CGP-E021]`, naming the first larger type, `Vec<u32>`, and the help names `ShowWrapped`, the
//! provider that requires `CanShow` of it.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanShow {
    fn show(&self) -> String;
}
impl<__Context__> CanShow for __Context__
where
    __Context__: Shower<__Context__>,
{
    fn show(&self) -> String {
        __Context__::show(self)
    }
}
pub trait Shower<__Context__>: IsProviderFor<ShowerComponent, __Context__, ()> {
    fn show(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Shower<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ShowerComponent>
        + IsProviderFor<ShowerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<ShowerComponent>>::Delegate: Shower<__Context__>,
{
    fn show(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<ShowerComponent>>::Delegate::show(__context__)
    }
}
///Component marker for [`CanShow`].
pub struct ShowerComponent;
impl<__Context__> Shower<__Context__> for UseContext
where
    __Context__: CanShow,
{
    fn show(__context__: &__Context__) -> String {
        __Context__::show(__context__)
    }
}
impl<__Context__> IsProviderFor<ShowerComponent, __Context__, ()> for UseContext
where
    __Context__: CanShow,
{}
impl<__Context__, __Components__, __Path__> Shower<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Shower<__Context__>,
{
    fn show(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::show(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ShowerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ShowerComponent, __Context__, ()> + Shower<__Context__>,
{}
impl<__Context__> Shower<__Context__> for ShowWrapped
where
    Vec<__Context__>: CanShow,
{
    fn show(__context__: &__Context__) -> String {
        String::new()
    }
}
impl<__Context__> IsProviderFor<ShowerComponent, __Context__, ()> for ShowWrapped
where
    Vec<__Context__>: CanShow,
{}
///Provider `ShowWrapped` for [`Shower`].
pub struct ShowWrapped;
impl DelegateComponent<ShowerComponent> for u32 {
    type Delegate = ShowWrapped;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ShowerComponent, __Context__, __Params__> for u32
where
    ShowWrapped: IsProviderFor<ShowerComponent, __Context__, __Params__>,
{}
impl<T> DelegateComponent<ShowerComponent> for Vec<T> {
    type Delegate = ShowWrapped;
}
impl<
    T,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ShowerComponent, __Context__, __Params__> for Vec<T>
where
    ShowWrapped: IsProviderFor<ShowerComponent, __Context__, __Params__>,
{}
trait __Checku32<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __Checku32<ShowerComponent, ()> for u32 {}
fn main() {}
