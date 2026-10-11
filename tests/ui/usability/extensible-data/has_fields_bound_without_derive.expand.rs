#![feature(prelude_import)]
//! A provider bound on the context's `HasFields` shape, wired on a context without the derive.
//!
//! The check resolves to a `[CGP-E201]` leaf `App: HasFields` with no `#[derive(HasFields)]` help,
//! unlike the `HasField` leaves, which carry their derive help.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanDescribe {
    fn describe(&self) -> String;
}
impl<__Context__> CanDescribe for __Context__
where
    __Context__: Describer<__Context__>,
{
    fn describe(&self) -> String {
        __Context__::describe(self)
    }
}
pub trait Describer<__Context__>: IsProviderFor<DescriberComponent, __Context__, ()> {
    fn describe(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Describer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DescriberComponent>
        + IsProviderFor<DescriberComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DescriberComponent,
    >>::Delegate: Describer<__Context__>,
{
    fn describe(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            DescriberComponent,
        >>::Delegate::describe(__context__)
    }
}
pub struct DescriberComponent;
impl<__Context__> Describer<__Context__> for UseContext
where
    __Context__: CanDescribe,
{
    fn describe(__context__: &__Context__) -> String {
        __Context__::describe(__context__)
    }
}
impl<__Context__> IsProviderFor<DescriberComponent, __Context__, ()> for UseContext
where
    __Context__: CanDescribe,
{}
impl<__Context__, __Components__, __Path__> Describer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Describer<__Context__>,
{
    fn describe(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::describe(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DescriberComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DescriberComponent, __Context__, ()>
        + Describer<__Context__>,
{}
impl<__Context__> Describer<__Context__> for DescribeNamed
where
    __Context__: HasFields<
        Fields = Struct! { name: String },
    >,
{
    fn describe(__context__: &__Context__) -> String {
        "named".into()
    }
}
impl<__Context__> IsProviderFor<DescriberComponent, __Context__, ()> for DescribeNamed
where
    __Context__: HasFields<
        Fields = Struct! { name: String },
    >,
{}
pub struct DescribeNamed;
pub struct App {
    pub name: String,
}
impl HasField<Symbol!("name")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
impl DelegateComponent<DescriberComponent> for App {
    type Delegate = DescribeNamed;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<DescriberComponent, __Context__, __Params__> for App
where
    DescribeNamed: IsProviderFor<DescriberComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<DescriberComponent, ()> for App {}
fn main() {}
