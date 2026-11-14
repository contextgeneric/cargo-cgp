#![feature(prelude_import)]
//! A provider header missing its key beside headers that name the wrong trait.
//!
//! A header naming the consumer trait (`impl CanGreet`) or a non-CGP trait (`impl Wave`) also fails
//! with an `E0425` for the `…Component` it derives, but the mistake there is the trait, reported as
//! `[CGP-E032]` and `[CGP-E033]`, with that `E0425` dropped. Only the header naming the provider
//! trait without its key is `[CGP-E041]`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md),
//! and wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod components {
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
    ///Provider trait for [`CanGreet`]. A provider imports this trait's component key, [`GreeterComponent`], beside it.
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
    pub trait Wave {
        fn wave(&self) -> String;
    }
}
pub mod providers {
    use cgp::prelude::*;
    use crate::components::{CanGreet, Greeter, Wave};
    impl<__Context__> Greeter<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) -> String {
            "Hello".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
    ///Provider `GreetHello` for [`Greeter`].
    pub struct GreetHello;
    impl<__Context__> CanGreet<__Context__> for GreetConsumer {
        fn greet(__context__: &__Context__) -> String {
            "Hi".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<CanGreetComponent, __Context__, ()>
    for GreetConsumer {}
    ///Provider `GreetConsumer` for [`CanGreet`].
    pub struct GreetConsumer;
    impl<__Context__> Wave<__Context__> for WaveHand {
        fn wave(__context__: &__Context__) -> String {
            "wave".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<WaveComponent, __Context__, ()> for WaveHand {}
    ///Provider `WaveHand` for [`Wave`].
    pub struct WaveHand;
}
fn main() {}
