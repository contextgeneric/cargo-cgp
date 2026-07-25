#![feature(prelude_import)]
//! The constrained-key delegation failure in its canonical core-CGP form: wiring a component to
//! `PipeHandlers<Product![]>`, an *empty* pipeline.
//!
//! `PipeHandlers<Providers>`'s own `delegate_components!` (in `cgp-handler`) carries a constrained
//! generic list — `Providers: ComposeProviders<Provider = Provider>` — so its
//! `DelegateComponent<Component> for PipeHandlers<Providers>` impl applies only when `Providers`
//! composes. `ComposeProviders` is defined for a non-empty `Cons` list but not for the empty `Nil`,
//! so `PipeHandlers<Product![]>` (whose list is `Nil`) has no working delegation. `App` wires
//! `GreeterComponent` to it — `PipeHandlers`'s delegation is generic over the component, so any
//! component routes through it — and the check fails because the delegate entry's constrained key is
//! unsatisfiable, not because the component is unwired. The resolver should lead with the real
//! composition bound rather than the `IsProviderFor`/`DelegateComponent` scaffolding.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::handler::PipeHandlers;
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
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = PipeHandlers<Nil>;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    PipeHandlers<Nil>: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
fn main() {}
