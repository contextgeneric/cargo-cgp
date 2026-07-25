#![feature(prelude_import)]
//! A component delegation that fails because the `DelegateComponent` impl carries a **constrained
//! key** whose `where`-clause is unsatisfied — the shape `PipeHandlers<Providers>` produces (its
//! `delegate_components!` generic list is `Providers: ComposeProviders<Provider = Provider>`, so an
//! un-composable `Providers` makes the delegation itself fail).
//!
//! `PickFirstProvider<List>` is such a dispatcher: its wiring delegates every component to the
//! provider its `List` parameter reduces to, but only when `List: PickFirst` holds — and `PickFirst`
//! is implemented for a non-empty `Cons`, never for the empty `Nil`. `App` wires `GreeterComponent`
//! to `PickFirstProvider<Product![]>`, whose `Nil` list has no `PickFirst` impl, so the generated
//! `DelegateComponent<GreeterComponent> for PickFirstProvider<Nil>` impl cannot apply. The delegate
//! entry *exists* but its own bound is unmet — distinct from a component wired nowhere.
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
impl<__Context__> Greeter<__Context__> for HelloGreeter {
    fn greet(__context__: &__Context__) -> String {
        "hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for HelloGreeter {}
pub struct HelloGreeter;
/// A dispatcher provider that forwards a component to whatever provider its `List` reduces to.
pub struct PickFirstProvider<List>(pub PhantomData<List>);
/// The reduction: a non-empty list yields its head provider. `Nil` has no impl, so an empty list
/// cannot reduce.
pub trait PickFirst {
    type Provider;
}
impl<Head, Tail> PickFirst for Cons<Head, Tail> {
    type Provider = Head;
}
impl<
    Component,
    Provider,
    List: PickFirst<Provider = Provider>,
> DelegateComponent<Component> for PickFirstProvider<List> {
    type Delegate = Provider;
}
impl<
    Component,
    Provider,
    List: PickFirst<Provider = Provider>,
    __Context__,
    __Params__,
> IsProviderFor<Component, __Context__, __Params__> for PickFirstProvider<List>
where
    Provider: IsProviderFor<Component, __Context__, __Params__>,
{}
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = PickFirstProvider<Nil>;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    PickFirstProvider<Nil>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
