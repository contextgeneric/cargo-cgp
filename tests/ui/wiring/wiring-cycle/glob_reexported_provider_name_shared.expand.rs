#![feature(prelude_import)]
//! Two providers named `EvalAdd`, one glob-imported, each named in a cycle's help.
//!
//! One `EvalAdd` is defined in the private module `providers::eval::add` and reaches the crate root
//! through glob re-exports; the other is `eval::add::EvalAdd`, under a root module of the same
//! name. Each provides `Greeter` by requiring `CanGreet` on the context, so `App`, wired to the
//! root one, and `Other`, wired to the glob-imported one, both loop back (`E0275`). Neither module
//! name is unique, so the compiler prints each provider's full path. The bare name `EvalAdd` means
//! the glob-imported provider here, so `Other`'s `[CGP-E020]` help names it bare, and `App`'s keeps
//! `eval::add::EvalAdd`, though that path is also the tail of the other provider's.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use crate::components::*;
use crate::providers::*;
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
}
pub mod providers {
    mod eval {
        mod add {
            use cgp::prelude::*;
            use crate::components::*;
            impl<__Context__> Greeter<__Context__> for EvalAdd
            where
                __Context__: CanGreet,
            {
                fn greet(__context__: &__Context__) {
                    __context__.greet()
                }
            }
            impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()>
            for EvalAdd
            where
                __Context__: CanGreet,
            {}
            ///Provider `EvalAdd` for [`Greeter`].
            pub struct EvalAdd;
        }
        pub use add::*;
    }
    pub use eval::*;
}
pub mod eval {
    pub mod add {
        use cgp::prelude::*;
        use crate::components::*;
        impl<__Context__> Greeter<__Context__> for EvalAdd
        where
            __Context__: CanGreet,
        {
            fn greet(__context__: &__Context__) {
                __context__.greet()
            }
        }
        impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for EvalAdd
        where
            __Context__: CanGreet,
        {}
        ///Provider `EvalAdd` for [`Greeter`].
        pub struct EvalAdd;
    }
}
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = eval::add::EvalAdd;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    eval::add::EvalAdd: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, ()> for App {}
pub struct Other;
impl DelegateComponent<GreeterComponent> for Other {
    type Delegate = EvalAdd;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Other
where
    EvalAdd: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckOther<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckOther<GreeterComponent, ()> for Other {}
fn main() {}
