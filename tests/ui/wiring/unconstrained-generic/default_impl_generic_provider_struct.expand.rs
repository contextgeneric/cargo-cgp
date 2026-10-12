#![feature(prelude_import)]
//! `#[default_impl]` on the impl of a provider struct that is itself generic over the dispatched
//! type.
//!
//! The registration copies the impl's parameter `T` and names the provider `ShowWith<T>`, but its
//! key `String` and the table do not use `T`, so the compiler rejects it with `E0207`. The tool
//! reports `[CGP-E018]`, as for a provider struct that does not use the parameter, with a help that
//! registers the provider at the key: `#[cgp_impl(ShowWith<String>)]` with `String` in place of `T`.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::fmt::Display;
use core::marker::PhantomData;
use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}
impl<__Context__, T> CanShow<T> for __Context__
where
    __Context__: ShowImpl<__Context__, T>,
{
    fn show(&self, value: &T) -> String {
        __Context__::show(self, value)
    }
}
pub trait ShowImpl<__Context__, T>: IsProviderFor<ShowImplComponent, __Context__, (T)> {
    fn show(__context__: &__Context__, value: &T) -> String;
}
impl<__Provider__, __Context__, T> ShowImpl<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<ShowImplComponent>
        + IsProviderFor<ShowImplComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        ShowImplComponent,
    >>::Delegate: ShowImpl<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Provider__ as DelegateComponent<
            ShowImplComponent,
        >>::Delegate::show(__context__, value)
    }
}
pub struct ShowImplComponent;
impl<__Context__, T> ShowImpl<__Context__, T> for UseContext
where
    __Context__: CanShow<T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        __Context__::show(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<ShowImplComponent, __Context__, (T)> for UseContext
where
    __Context__: CanShow<T>,
{}
impl<__Context__, T, __Components__, __Path__> ShowImpl<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: ShowImpl<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::show(__context__, value)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<ShowImplComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<ShowImplComponent, __Context__, (T)>
        + ShowImpl<__Context__, T>,
{}
pub struct ShowWith<T>(pub PhantomData<T>);
impl<__Context__, T: Display> ShowImpl<__Context__, T> for ShowWith<T> {
    fn show(__context__: &__Context__, value: &T) -> String {
        ::alloc::__export::must_use({ ::alloc::fmt::format(format_args!("{0}", value)) })
    }
}
impl<__Context__, T: Display> IsProviderFor<ShowImplComponent, __Context__, (T)>
for ShowWith<T> {}
impl<T: Display, __Components__> DefaultImpls1<ShowImplComponent, __Components__>
for String {
    type Delegate = ShowWith<T>;
}
fn main() {}
