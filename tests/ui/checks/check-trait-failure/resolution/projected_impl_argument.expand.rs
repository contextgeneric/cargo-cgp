#![feature(prelude_import)]
//! Two blanket impls of one trait that differ only in a projected argument, and the walk choosing
//! between them. `Pick<<u8 as Assoc>::Out>` is `Pick<A>` and `Pick<<u16 as Assoc>::Out>` is
//! `Pick<B>`, so the two impls are coherent, and each requires a different field. The provider needs
//! `Self: Pick<B>`, and the context has neither field, so the root cause is the missing `beta`
//! that the second impl requires.
//!
//! Property: the walk matches an obligation against an impl only where the impl's trait ref, its
//! projections reduced, equals the obligation's. An impl matched with its projection unreduced
//! relates `<u8 as Assoc>::Out` to `B` as a goal left for later, so the first impl looks like a
//! match and the cause would name `alpha`, the field the first impl requires, which `Pick<B>` does
//! not need.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct A;
pub struct B;
pub trait Assoc {
    type Out;
}
impl Assoc for u8 {
    type Out = A;
}
impl Assoc for u16 {
    type Out = B;
}
pub trait Pick<T> {}
impl<Ctx> Pick<<u8 as Assoc>::Out> for Ctx
where
    Ctx: HasField<Symbol!("alpha"), Value = u32>,
{}
impl<Ctx> Pick<<u16 as Assoc>::Out> for Ctx
where
    Ctx: HasField<Symbol!("beta"), Value = u32>,
{}
pub trait CanGreet {
    fn greet(&self);
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__);
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) {
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
    fn greet(__context__: &__Context__) {
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
    fn greet(__context__: &__Context__) {
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
impl<__Context__> Greeter<__Context__> for GreetPicked
where
    __Context__: Pick<B>,
{
    fn greet(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetPicked
where
    __Context__: Pick<B>,
{}
pub struct GreetPicked;
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
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetPicked;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    GreetPicked: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
