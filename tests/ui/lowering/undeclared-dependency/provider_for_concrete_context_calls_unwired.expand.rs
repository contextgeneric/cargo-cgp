#![feature(prelude_import)]
//! A provider written for one context, `impl Greeter for Person`, whose body calls a consumer trait
//! `Person` does not wire.
//!
//! The provider's context is the concrete `Person`, not a generated `__Context__`, so the call is not
//! an undeclared dependency: `#[uses(CanWave)]` would only restate a bound `Person` fails. The tool
//! resolves the call through the call-site anchor instead, to the missing wiring of
//! `WaverComponent` on `Person`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
///Component marker for [`CanGreet`].
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
pub trait CanWave {
    fn wave(&self) -> String;
}
impl<__Context__> CanWave for __Context__
where
    __Context__: Waver<__Context__>,
{
    fn wave(&self) -> String {
        __Context__::wave(self)
    }
}
pub trait Waver<__Context__>: IsProviderFor<WaverComponent, __Context__, ()> {
    fn wave(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Waver<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<WaverComponent>
        + IsProviderFor<WaverComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<WaverComponent>>::Delegate: Waver<__Context__>,
{
    fn wave(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<WaverComponent>>::Delegate::wave(__context__)
    }
}
///Component marker for [`CanWave`].
pub struct WaverComponent;
impl<__Context__> Waver<__Context__> for UseContext
where
    __Context__: CanWave,
{
    fn wave(__context__: &__Context__) -> String {
        __Context__::wave(__context__)
    }
}
impl<__Context__> IsProviderFor<WaverComponent, __Context__, ()> for UseContext
where
    __Context__: CanWave,
{}
impl<__Context__, __Components__, __Path__> Waver<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Waver<__Context__>,
{
    fn wave(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::wave(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<WaverComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<WaverComponent, __Context__, ()> + Waver<__Context__>,
{}
pub struct Person {
    pub name: String,
}
impl HasField<Symbol!("name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
impl Greeter<Person> for GreetPerson {
    fn greet(__person__: &Person) -> String {
        __person__.wave()
    }
}
impl IsProviderFor<GreeterComponent, Person, ()> for GreetPerson {}
///Provider `GreetPerson` for [`Greeter`].
pub struct GreetPerson;
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetPerson;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetPerson: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {}
