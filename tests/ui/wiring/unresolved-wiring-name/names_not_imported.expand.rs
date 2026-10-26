#![feature(prelude_import)]
//! Wiring names and a `#[cgp_impl]` provider that are declared, in another module or in a
//! dependency, but not imported where they are written.
//!
//! `GreetHello` and `GreetBye` are declared in `providers`, `Square` in `shapes`, and `RaiseFrom` in
//! `cgp::extra::error`, and none is imported at the crate root, where `GreetBye` is also wired as
//! `Robot`'s provider. rustc reports an `E0425` for each name, with a suggestion to import it where
//! it finds one, which it does for the crate's own types.
//! Each name is declared, so the tool does not call any of them undeclared: it leaves every `E0425`
//! as rustc wrote it, the import suggestions included.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::prelude::*;
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
pub mod providers {
    use cgp::prelude::*;
    use super::{Greeter, GreeterComponent};
    impl<__Context__> Greeter<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) {}
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
    pub struct GreetHello;
    pub struct GreetBye;
}
pub mod shapes {
    pub struct Square;
}
impl<__Context__> Greeter<__Context__> for GreetBye {
    fn greet(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetBye {}
pub struct Person;
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
pub struct Robot;
impl DelegateComponent<GreeterComponent> for Robot {
    type Delegate = GreetBye;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Robot
where
    GreetBye: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
pub struct App;
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ErrorRaiserComponent> for App {
    type Delegate = RaiseFrom;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for App
where
    RaiseFrom: IsProviderFor<ErrorRaiserComponent, __Context__, __Params__>,
{}
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@app.Square)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@app.Square),
    >: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<Square, __Wildcard__>>> for App {
    type Delegate = providers::GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<Square, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    providers::GreetHello: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<Square, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
