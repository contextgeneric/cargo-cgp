#![feature(prelude_import)]
//! A wrapper impl on a context whose supertrait and own `where` bound fail for different causes.
//!
//! `CanGreetTwice` has the consumer trait `CanGreet` as its supertrait, which fails because
//! `Person` lacks the `name` field `GreetHello` reads. The impl also requires `Person: CanWave`, and
//! `Person` wires no waver. rustc reports the supertrait at the impl's `Self` type and the bound at
//! the bound. The tool reports each cause on its own caret: the missing waver wiring on the bound,
//! which the impl-site anchor reads as the consumer obligation it names, and the missing field on
//! the impl's `Self` type, from the supertrait.
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
impl<__Context__> Greeter<__Context__> for GreetHello
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) -> String {
        __context__.name().to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasName,
{}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
pub struct Person {}
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHello;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
pub trait CanGreetTwice: CanGreet {}
impl CanGreetTwice for Person
where
    Person: CanWave,
{}
fn main() {}
