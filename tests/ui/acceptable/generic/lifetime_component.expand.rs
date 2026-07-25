#![feature(prelude_import)]
//! A component carrying a *lifetime* parameter, failing its check on a missing field.
//!
//! `#[cgp_component]` on a trait with a lifetime keeps the lifetime ahead of the
//! context in the provider trait (`ReferenceGetter<'a, Context, T>`) and lifts it
//! into `Life<'a>` in the check entry's params tuple. The resolver must rebuild the
//! consumer obligation with the lifetime restored to its region slot — not spread
//! `Life<'a>` in as a type — and label the provider node without assuming the
//! context is the trait's first argument.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasReference<'a, T: 'a + ?Sized> {
    fn get_reference(&self) -> &'a T;
}
impl<'a, __Context__, T: 'a + ?Sized> HasReference<'a, T> for __Context__
where
    __Context__: ReferenceGetter<'a, __Context__, T>,
{
    fn get_reference(&self) -> &'a T {
        __Context__::get_reference(self)
    }
}
pub trait ReferenceGetter<
    'a,
    __Context__,
    T: 'a + ?Sized,
>: IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, T)> {
    fn get_reference(__context__: &__Context__) -> &'a T;
}
impl<'a, __Provider__, __Context__, T: 'a + ?Sized> ReferenceGetter<'a, __Context__, T>
for __Provider__
where
    __Provider__: DelegateComponent<ReferenceGetterComponent>
        + IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, T)>,
    <__Provider__ as DelegateComponent<
        ReferenceGetterComponent,
    >>::Delegate: ReferenceGetter<'a, __Context__, T>,
{
    fn get_reference(__context__: &__Context__) -> &'a T {
        <__Provider__ as DelegateComponent<
            ReferenceGetterComponent,
        >>::Delegate::get_reference(__context__)
    }
}
pub struct ReferenceGetterComponent;
impl<'a, __Context__, T: 'a + ?Sized> ReferenceGetter<'a, __Context__, T> for UseContext
where
    __Context__: HasReference<'a, T>,
{
    fn get_reference(__context__: &__Context__) -> &'a T {
        __Context__::get_reference(__context__)
    }
}
impl<
    'a,
    __Context__,
    T: 'a + ?Sized,
> IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, T)> for UseContext
where
    __Context__: HasReference<'a, T>,
{}
impl<
    'a,
    __Context__,
    T: 'a + ?Sized,
    __Components__,
    __Path__,
> ReferenceGetter<'a, __Context__, T> for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: ReferenceGetter<'a, __Context__, T>,
{
    fn get_reference(__context__: &__Context__) -> &'a T {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::get_reference(__context__)
    }
}
impl<
    'a,
    __Context__,
    T: 'a + ?Sized,
    __Components__,
    __Path__,
> IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: ReferenceGetter<'a, __Context__, T>,
{}
pub trait HasName {
    fn name(&self) -> &String;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &String {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>)
    }
}
impl<'a, __Context__> ReferenceGetter<'a, __Context__, str> for GetReference
where
    __Context__: HasName,
{
    fn get_reference(__context__: &__Context__) -> &'a str {
        let _ = __context__.name();
        ""
    }
}
impl<
    'a,
    __Context__,
> IsProviderFor<ReferenceGetterComponent, __Context__, (Life<'a>, str)> for GetReference
where
    __Context__: HasName,
{}
pub struct GetReference;
pub struct App<'a> {
    pub value: &'a str,
}
impl<'a> HasField<Symbol!("value")> for App<'a> {
    type Value = &'a str;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("value")>,
    ) -> &Self::Value {
        &self.value
    }
}
impl<'a> HasFieldMut<Symbol!("value")> for App<'a> {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("value")>,
    ) -> &mut Self::Value {
        &mut self.value
    }
}
impl<'a> DelegateComponent<ReferenceGetterComponent> for App<'a> {
    type Delegate = GetReference;
}
impl<
    'a,
    __Context__,
    __Params__,
> IsProviderFor<ReferenceGetterComponent, __Context__, __Params__> for App<'a>
where
    GetReference: IsProviderFor<ReferenceGetterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl<'a> __CheckApp<ReferenceGetterComponent, (Life<'a>, str)> for App<'a> {}
fn main() {}
