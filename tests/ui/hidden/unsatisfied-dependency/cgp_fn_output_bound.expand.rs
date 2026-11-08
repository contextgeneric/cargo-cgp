#![feature(prelude_import)]
//! A `#[cgp_fn]` whose generic output is pinned by a provider bound, the shape of a JSON helper.
//! The call leaves the output to inference, so the call-site anchor seeds it as a placeholder and
//! still reaches the provider's missing field.
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
///Component marker for [`CanLoad`].
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
impl<Context, T> Loader<Context, T> for LoadVia
where
    T: Default,
    Context: HasName,
{
    type Output = T;
    fn load(context: &Context) -> T {
        let _ = context.name();
        T::default()
    }
}
impl<Context, T> IsProviderFor<LoaderComponent, Context, (T)> for LoadVia
where
    T: Default,
    Context: HasName,
{}
///Provider `LoadVia` for [`Loader`].
pub struct LoadVia;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        ::core::convert::AsRef::as_ref(
            HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("name")>),
        )
    }
}
///The trait for the `fetch_value` operation.
pub trait CanFetchValue<T> {
    ///The `fetch_value` operation.
    fn fetch_value(&self) -> T;
}
impl<__Context__, T> CanFetchValue<T> for __Context__
where
    LoadVia: Loader<Self, T, Output = T>,
{
    fn fetch_value(&self) -> T {
        LoadVia::load(self)
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
fn main() {
    let app = App { other: String::new() };
    let _: u32 = app.fetch_value();
}
