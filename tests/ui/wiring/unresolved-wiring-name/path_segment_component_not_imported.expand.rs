#![feature(prelude_import)]
//! An `@`-path segment naming a component that is not in scope.
//!
//! The segment `GreeterComponent` is a component of another module, so it needs its import. The
//! compiler's suggestions include a similarly named component of another kind. The tool words the
//! segment as `[CGP-E041]`, the same as a component written as a key, with the import path.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
mod parts {
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
    impl<__Context__> Greeter<__Context__> for GreetHello {
        fn greet(__context__: &__Context__) -> String {
            "hello".into()
        }
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
    pub struct GreetHello;
}
use parts::GreetHello;
pub struct App;
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>>
for App {
    type Delegate = GreetHello;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetHello: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<GreeterComponent, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
