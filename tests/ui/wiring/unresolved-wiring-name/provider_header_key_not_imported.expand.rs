#![feature(prelude_import)]
//! Provider headers that import a component's provider trait from another module but not its key.
//!
//! `#[cgp_impl]` and `#[cgp_provider]` name the component after the provider trait, as
//! `GreeterComponent`, so the module must import that key beside `Greeter`. Without it, each header
//! is an `E0425` "cannot find type `GreeterComponent`" on the trait's name, which the tool reshapes
//! into `[CGP-E041]` with the import, `use crate::components::GreeterComponent;`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
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
}
pub mod providers {
    use cgp::prelude::*;
    use crate::components::Greeter;
    impl<__Context__> Greeter<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) -> String {
            "Hello".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
    ///Provider `GreetHello` for [`Greeter`].
    pub struct GreetHello;
    pub struct GreetRaw;
    impl<Context> Greeter<Context> for GreetRaw {
        fn greet(_context: &Context) -> String {
            "Hi".to_owned()
        }
    }
    impl<Context> IsProviderFor<GreeterComponent, Context, ()> for GreetRaw {}
}
fn main() {}
