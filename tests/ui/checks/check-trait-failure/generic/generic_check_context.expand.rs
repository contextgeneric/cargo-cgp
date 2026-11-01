#![feature(prelude_import)]
//! A check that carries a generic parameter, on a context missing a field.
//!
//! `App<T>` wires `GreeterComponent`, and `GreetName` reads a `name` field the struct lacks. The
//! resolver solves under an empty parameter environment, where the solver does not prove a wiring
//! lookup that names the check's own `T`, so walking the entry would report the wired component as
//! missing. It declines instead, and the text rewrite keeps rustc's headline over the
//! `missing field` help.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self) -> String;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> String {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) -> String {
        __Context__::greet(__context__)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseContext
where
    __Context__: CanGreet,
{}
impl<__Context__, __Components__, __Path__> Greeter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, ()>
        + Greeter<__Context__>,
{}
impl<__Context__> Greeter<__Context__> for GreetName
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn greet(__context__: &__Context__) -> String {
        let name: &str = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetName
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{}
pub struct GreetName;
pub struct App<T> {
    pub other: PhantomData<T>,
}
impl<T> HasField<Symbol!("other")> for App<T> {
    type Value = PhantomData<T>;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &Self::Value {
        &self.other
    }
}
impl<T> HasFieldMut<Symbol!("other")> for App<T> {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("other")>,
    ) -> &mut Self::Value {
        &mut self.other
    }
}
impl<T> DelegateComponent<GreeterComponent> for App<T> {
    type Delegate = GreetName;
}
impl<
    T,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App<T>
where
    GreetName: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl<T> __CheckApp<GreeterComponent, ()> for App<T> {}
fn main() {}
