#![feature(prelude_import)]
//! A wiring key naming a component defined in another module of the crate, without its import.
//!
//! `GreeterComponent` is declared in `components`, but the wiring module does not import it, so the
//! key is an `E0425` "cannot find type". rustc suggests the right import,
//! `use crate::components::GreeterComponent;`, beside a similarly named `ComputerComponent`, which
//! is a different component.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod components {
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
pub mod wiring {
    use cgp::prelude::*;
    use crate::components::GreetHello;
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
}
fn main() {}
