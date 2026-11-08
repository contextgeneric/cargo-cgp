#![feature(prelude_import)]
//! A `#[uses]` dependency that gives its consumer trait one generic argument too many.
//!
//! `CanFoo<T>` takes one argument, and `#[uses(CanFoo<u32, u64>)]` gives it two. The error is a
//! plain arity slip (`E0107`), and rustc's message names it with the fix. The bound sits on the
//! generated context, not on an inner provider, so it is no consumer trait in an inner-provider
//! bound and must stay exactly as rustc prints it.
//!
//! CGP error class: none; plain Rust the tool passes through.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanFoo<T> {
    fn foo(&self, value: T) -> u32;
}
impl<__Context__, T> CanFoo<T> for __Context__
where
    __Context__: Fooer<__Context__, T>,
{
    fn foo(&self, value: T) -> u32 {
        __Context__::foo(self, value)
    }
}
pub trait Fooer<__Context__, T>: IsProviderFor<FooerComponent, __Context__, (T)> {
    fn foo(__context__: &__Context__, value: T) -> u32;
}
impl<__Provider__, __Context__, T> Fooer<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<FooerComponent>
        + IsProviderFor<FooerComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<FooerComponent>>::Delegate: Fooer<__Context__, T>,
{
    fn foo(__context__: &__Context__, value: T) -> u32 {
        <__Provider__ as DelegateComponent<
            FooerComponent,
        >>::Delegate::foo(__context__, value)
    }
}
///Component marker for [`CanFoo`].
pub struct FooerComponent;
impl<__Context__, T> Fooer<__Context__, T> for UseContext
where
    __Context__: CanFoo<T>,
{
    fn foo(__context__: &__Context__, value: T) -> u32 {
        __Context__::foo(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<FooerComponent, __Context__, (T)> for UseContext
where
    __Context__: CanFoo<T>,
{}
impl<__Context__, T, __Components__, __Path__> Fooer<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: Fooer<__Context__, T>,
{
    fn foo(__context__: &__Context__, value: T) -> u32 {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::foo(__context__, value)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<FooerComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<FooerComponent, __Context__, (T)>
        + Fooer<__Context__, T>,
{}
pub trait CanBar {
    fn bar(&self) -> u32;
}
impl<__Context__> CanBar for __Context__
where
    __Context__: Barer<__Context__>,
{
    fn bar(&self) -> u32 {
        __Context__::bar(self)
    }
}
pub trait Barer<__Context__>: IsProviderFor<BarerComponent, __Context__, ()> {
    fn bar(__context__: &__Context__) -> u32;
}
impl<__Provider__, __Context__> Barer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<BarerComponent>
        + IsProviderFor<BarerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<BarerComponent>>::Delegate: Barer<__Context__>,
{
    fn bar(__context__: &__Context__) -> u32 {
        <__Provider__ as DelegateComponent<BarerComponent>>::Delegate::bar(__context__)
    }
}
///Component marker for [`CanBar`].
pub struct BarerComponent;
impl<__Context__> Barer<__Context__> for UseContext
where
    __Context__: CanBar,
{
    fn bar(__context__: &__Context__) -> u32 {
        __Context__::bar(__context__)
    }
}
impl<__Context__> IsProviderFor<BarerComponent, __Context__, ()> for UseContext
where
    __Context__: CanBar,
{}
impl<__Context__, __Components__, __Path__> Barer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Barer<__Context__>,
{
    fn bar(__context__: &__Context__) -> u32 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::bar(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<BarerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<BarerComponent, __Context__, ()> + Barer<__Context__>,
{}
impl<__Context__> Barer<__Context__> for BarFromFoo
where
    __Context__: CanFoo<u32, u64>,
{
    fn bar(__context__: &__Context__) -> u32 {
        __context__.foo(1)
    }
}
impl<__Context__> IsProviderFor<BarerComponent, __Context__, ()> for BarFromFoo
where
    __Context__: CanFoo<u32, u64>,
{}
///Provider `BarFromFoo` for [`Barer`].
pub struct BarFromFoo;
fn main() {}
