#![feature(prelude_import)]
//! Two `#[default_impl]` registrations of a provider struct that is itself generic over the impl's
//! parameter, which `[CGP-E018]` cannot write the fix for: one at a path key
//! (`@app.ShowImplComponent`), whose key gives no type to put in the parameter's place, and one of
//! `ShowPair<T, U>` at `String`, which would still name `U`. Each registration copies the provider
//! impl's parameters, so each parameter neither the key nor the table uses is unconstrained (E0207).
//!
//! The tool leaves the three errors as rustc wrote them. A registration is not a wiring entry, so the
//! `[CGP-E019]` detector, tried after `[CGP-E018]`, skips an impl whose parameter a provider impl
//! also declares; the shape is an open issue of the class.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
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
impl<__Components__> AppNamespace<__Components__> for ShowImplComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.ShowImplComponent)>;
}
pub struct __AppNamespaceComponents;
pub trait AppNamespace<__Table__> {
    type Delegate;
}
pub struct ShowWith<T>(pub PhantomData<T>);
impl<__Context__, T: Display> ShowImpl<__Context__, T> for ShowWith<T> {
    fn show(__context__: &__Context__, value: &T) -> String {
        ::alloc::__export::must_use({ ::alloc::fmt::format(format_args!("{0}", value)) })
    }
}
impl<__Context__, T: Display> IsProviderFor<ShowImplComponent, __Context__, (T)>
for ShowWith<T> {}
impl<T: Display, __Components__> AppNamespace<__Components__>
for Path!(@app.ShowImplComponent) {
    type Delegate = ShowWith<T>;
}
pub struct ShowPair<T, U>(pub PhantomData<(T, U)>);
impl<__Context__, T: Display, U> ShowImpl<__Context__, T> for ShowPair<T, U> {
    fn show(__context__: &__Context__, value: &T) -> String {
        ::alloc::__export::must_use({ ::alloc::fmt::format(format_args!("{0}", value)) })
    }
}
impl<__Context__, T: Display, U> IsProviderFor<ShowImplComponent, __Context__, (T)>
for ShowPair<T, U> {}
impl<T: Display, U, __Components__> DefaultImpls1<ShowImplComponent, __Components__>
for String {
    type Delegate = ShowPair<T, U>;
}
fn main() {}
