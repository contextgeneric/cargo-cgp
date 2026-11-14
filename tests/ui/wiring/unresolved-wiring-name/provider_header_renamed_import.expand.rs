#![feature(prelude_import)]
//! Provider headers that name a provider trait imported under another name.
//!
//! With `use crate::components::{Greeter as G, GreeterComponent};`, the header `impl G` makes the
//! macro name the component `GComponent`, which nothing declares, so rustc reports "cannot find
//! type `GComponent`" with no suggestion and calls the correct `GreeterComponent` import unused.
//! The tool reshapes each into `[CGP-E070]`, naming the trait's real component and the attribute
//! form that names it, in the spelling each header gave its macro: `#[cgp_impl]` with and without
//! `new`, `#[cgp_provider]`, and `#[cgp_new_provider]`.
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
    use crate::components::{Greeter as G, GreeterComponent};
    impl<__Context__> G<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) -> String {
            "Hello".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<GComponent, __Context__, ()> for GreetHello {}
    ///Provider `GreetHello` for [`G`].
    pub struct GreetHello;
    pub struct GreetPlain;
    impl<__Context__> G<__Context__> for GreetPlain {
        fn greet(__context__: &__Context__) -> String {
            "Hey".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<GComponent, __Context__, ()> for GreetPlain {}
    pub struct GreetRaw;
    impl<Context> G<Context> for GreetRaw {
        fn greet(_context: &Context) -> String {
            "Hi".to_owned()
        }
    }
    impl<Context> IsProviderFor<GComponent, Context, ()> for GreetRaw {}
    #[allow(clippy::needless_lifetimes)]
    impl<Context> G<Context> for GreetNew {
        fn greet(_context: &Context) -> String {
            "Yo".to_owned()
        }
    }
    impl<Context> IsProviderFor<GComponent, Context, ()> for GreetNew {}
    ///Provider `GreetNew` for [`G`].
    pub struct GreetNew;
}
fn main() {}
