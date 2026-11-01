#![feature(prelude_import)]
//! The same method bound failing only on its associated type. No anchor reads it, so rustc's
//! `E0271` stays as written.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanLoad<T> {
    type Output;
    fn load(&self) -> Self::Output;
}
impl<__Context__, T> CanLoad<T> for __Context__
where
    __Context__: Loader<__Context__, T>,
{
    type Output = <__Context__ as Loader<__Context__, T>>::Output;
    fn load(&self) -> Self::Output {
        __Context__::load(self)
    }
}
pub trait Loader<__Context__, T>: IsProviderFor<LoaderComponent, __Context__, (T)> {
    type Output;
    fn load(__context__: &__Context__) -> Self::Output;
}
impl<__Provider__, __Context__, T> Loader<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<LoaderComponent>
        + IsProviderFor<LoaderComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        LoaderComponent,
    >>::Delegate: Loader<__Context__, T>,
{
    type Output = <<__Provider__ as DelegateComponent<
        LoaderComponent,
    >>::Delegate as Loader<__Context__, T>>::Output;
    fn load(__context__: &__Context__) -> Self::Output {
        <__Provider__ as DelegateComponent<LoaderComponent>>::Delegate::load(__context__)
    }
}
pub struct LoaderComponent;
impl<__Context__, T> Loader<__Context__, T> for UseContext
where
    __Context__: CanLoad<T>,
{
    type Output = <__Context__ as CanLoad<T>>::Output;
    fn load(__context__: &__Context__) -> Self::Output {
        __Context__::load(__context__)
    }
}
impl<__Context__, T> IsProviderFor<LoaderComponent, __Context__, (T)> for UseContext
where
    __Context__: CanLoad<T>,
{}
impl<__Context__, T, __Components__, __Path__> Loader<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: Loader<__Context__, T>,
{
    type Output = <<__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate as Loader<__Context__, T>>::Output;
    fn load(__context__: &__Context__) -> Self::Output {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::load(__context__)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<LoaderComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<LoaderComponent, __Context__, (T)>
        + Loader<__Context__, T>,
{}
impl<__Context__, T> Loader<__Context__, T> for LoadString {
    type Output = String;
    fn load(__context__: &__Context__) -> String {
        String::new()
    }
}
impl<__Context__, T> IsProviderFor<LoaderComponent, __Context__, (T)> for LoadString {}
pub struct LoadString;
pub trait CanFetch {
    fn fetch<T>(&self) -> T
    where
        Self: CanLoad<T, Output = T>;
}
impl<C> CanFetch for C {
    fn fetch<T>(&self) -> T
    where
        Self: CanLoad<T, Output = T>,
    {
        self.load()
    }
}
pub struct App;
impl DelegateComponent<LoaderComponent> for App {
    type Delegate = LoadString;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<LoaderComponent, __Context__, __Params__> for App
where
    LoadString: IsProviderFor<LoaderComponent, __Context__, __Params__>,
{}
fn main() {
    let app = App;
    let _: u32 = app.fetch::<u32>();
}
