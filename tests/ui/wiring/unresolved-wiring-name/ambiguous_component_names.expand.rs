#![feature(prelude_import)]
//! Wiring keys that could name either of two components, so the tool names neither.
//!
//! Modules `a` and `c` each define a component named `GreeterComponent`, and modules `a` and `b`
//! each define a consumer trait named `CanGreet`, with the components `GreeterComponent` and
//! `WelcomerComponent`. None is imported at the crate root. `Person` wires the key
//! `GreeterComponent`, which either module's component could be, and `Robot` the key
//! `CanGreetComponent`, guessed from a consumer trait's name, which could mean either `CanGreet`.
//! The tool offers no import and no component for either, and leaves each key's `E0425` as rustc
//! wrote it, dropping only rustc's repeat of the second at the same key. The first suggests
//! importing each `GreeterComponent`, beside a similarly named `ComputerComponent`, and the second
//! suggests the similarly named trait `CanUseComponent` and a type parameter, none of which fits a
//! wiring key.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub mod a {
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
    impl<__Context__> Greeter<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) {}
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
    pub struct GreetHello;
}
pub mod b {
    use cgp::prelude::*;
    pub trait CanGreet {
        fn greet(&self);
    }
    impl<__Context__> CanGreet for __Context__
    where
        __Context__: Welcomer<__Context__>,
    {
        fn greet(&self) {
            __Context__::greet(self)
        }
    }
    pub trait Welcomer<__Context__>: IsProviderFor<WelcomerComponent, __Context__, ()> {
        fn greet(__context__: &__Context__);
    }
    impl<__Provider__, __Context__> Welcomer<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<WelcomerComponent>
            + IsProviderFor<WelcomerComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            WelcomerComponent,
        >>::Delegate: Welcomer<__Context__>,
    {
        fn greet(__context__: &__Context__) {
            <__Provider__ as DelegateComponent<
                WelcomerComponent,
            >>::Delegate::greet(__context__)
        }
    }
    pub struct WelcomerComponent;
    impl<__Context__> Welcomer<__Context__> for UseContext
    where
        __Context__: CanGreet,
    {
        fn greet(__context__: &__Context__) {
            __Context__::greet(__context__)
        }
    }
    impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()> for UseContext
    where
        __Context__: CanGreet,
    {}
    impl<__Context__, __Components__, __Path__> Welcomer<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<__Path__>>::Delegate: Welcomer<__Context__>,
    {
        fn greet(__context__: &__Context__) {
            <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<WelcomerComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<WelcomerComponent, __Context__, ()>
            + Welcomer<__Context__>,
    {}
    impl<__Context__> Welcomer<__Context__> for WelcomeHello {
        fn greet(__context__: &__Context__) {}
    }
    impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()>
    for WelcomeHello {}
    pub struct WelcomeHello;
}
pub mod c {
    use cgp::prelude::*;
    pub trait CanHail {
        fn hail(&self);
    }
    impl<__Context__> CanHail for __Context__
    where
        __Context__: Greeter<__Context__>,
    {
        fn hail(&self) {
            __Context__::hail(self)
        }
    }
    pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
        fn hail(__context__: &__Context__);
    }
    impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<GreeterComponent>
            + IsProviderFor<GreeterComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate: Greeter<__Context__>,
    {
        fn hail(__context__: &__Context__) {
            <__Provider__ as DelegateComponent<
                GreeterComponent,
            >>::Delegate::hail(__context__)
        }
    }
    pub struct GreeterComponent;
    impl<__Context__> Greeter<__Context__> for UseContext
    where
        __Context__: CanHail,
    {
        fn hail(__context__: &__Context__) {
            __Context__::hail(__context__)
        }
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseContext
    where
        __Context__: CanHail,
    {}
    impl<__Context__, __Components__, __Path__> Greeter<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
    {
        fn hail(__context__: &__Context__) {
            <__Components__ as DelegateComponent<__Path__>>::Delegate::hail(__context__)
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
}
pub struct Person;
pub struct Robot;
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = a::GreetHello;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    a::GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<CanGreetComponent> for Robot {
    type Delegate = b::WelcomeHello;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<CanGreetComponent, __Context__, __Params__> for Robot
where
    b::WelcomeHello: IsProviderFor<CanGreetComponent, __Context__, __Params__>,
{}
fn main() {}
