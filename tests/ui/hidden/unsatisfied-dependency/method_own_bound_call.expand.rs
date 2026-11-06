#![feature(prelude_import)]
//! A method whose own `where` clause requires a consumer the context cannot satisfy. The bound-call
//! anchor reads a path call's function, not a method's bound, and `CanFetch` shows no CGP evidence of
//! its own, so the resolver declines and rustc's headline stays over the `missing field` help.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
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
impl<__Context__, T> Loader<__Context__, T> for LoadName
where
    T: Default,
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    type Output = T;
    fn load(__context__: &__Context__) -> T {
        let name: &str = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        let _ = name;
        T::default()
    }
}
impl<__Context__, T> IsProviderFor<LoaderComponent, __Context__, (T)> for LoadName
where
    T: Default,
    __Context__: HasField<Symbol!("name"), Value = String>,
{}
pub struct LoadName;
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
pub struct App {
    pub other: String,
}
impl HasField<Symbol!("other")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &Self::Value {
        &self.other
    }
}
impl HasFieldMut<Symbol!("other")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &mut Self::Value {
        &mut self.other
    }
}
impl DelegateComponent<LoaderComponent> for App {
    type Delegate = LoadName;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<LoaderComponent, __Context__, __Params__> for App
where
    LoadName: IsProviderFor<LoaderComponent, __Context__, __Params__>,
{}
fn main() {
    let app = App { other: String::new() };
    let _: u32 = app.fetch::<u32>();
}
