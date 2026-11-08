#![feature(prelude_import)]
//! A consumer method called where its trait is not imported.
//!
//! The error is plain Rust: the trait that provides `greet` is not in scope. The tool must print
//! what plain `cargo check` prints, short paths included (`struct Person`, not `lib::Person`).
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
mod lib {
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
    ///Component marker for [`CanGreet`].
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
    ///Provider `GreetHello` for [`Greeter`].
    pub struct GreetHello;
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
fn main() {
    lib::Person.greet();
}
