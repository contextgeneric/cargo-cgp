#![feature(prelude_import)]
//! The import `[CGP-E041]` offers for a component key that is not in scope, as the fix an editor
//! or `cargo fix` applies.
//!
//! A provider header and a wiring key each name a component the module does not import. The fix
//! inserts the `use` line where rustc puts its own import suggestion, before the module's first
//! `use` line, or at the start of a module that has none, followed by a blank line. It passes over
//! a `use` line with an attribute, which the inserted line would come between. A second,
//! unchanged part spans the name the error points at, so an editor offers the fix there. The fix
//! is machine-applicable when one public path reaches the key, as for a key of this crate,
//! `cgp`'s `ErrorTypeProviderComponent`, and `cgp_test_component_library`'s `WaverComponent`,
//! and only maybe correct when two do, as for `cgp_test_crate_a`'s `GreeterComponent`, which
//! `cgp_test_component_library` re-exports.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod components {
    use cgp::prelude::*;
    pub trait CanWelcome {
        fn welcome(&self) -> String;
    }
    impl<__Context__> CanWelcome for __Context__
    where
        __Context__: Welcomer<__Context__>,
    {
        fn welcome(&self) -> String {
            __Context__::welcome(self)
        }
    }
    ///Provider trait for [`CanWelcome`]. A provider imports this trait's component key, [`WelcomerComponent`], beside it.
    pub trait Welcomer<__Context__>: IsProviderFor<WelcomerComponent, __Context__, ()> {
        fn welcome(__context__: &__Context__) -> String;
    }
    impl<__Provider__, __Context__> Welcomer<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<WelcomerComponent>
            + IsProviderFor<WelcomerComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            WelcomerComponent,
        >>::Delegate: Welcomer<__Context__>,
    {
        fn welcome(__context__: &__Context__) -> String {
            <__Provider__ as DelegateComponent<
                WelcomerComponent,
            >>::Delegate::welcome(__context__)
        }
    }
    ///Component marker for [`CanWelcome`].
    pub struct WelcomerComponent;
    impl<__Context__> Welcomer<__Context__> for UseContext
    where
        __Context__: CanWelcome,
    {
        fn welcome(__context__: &__Context__) -> String {
            __Context__::welcome(__context__)
        }
    }
    impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()> for UseContext
    where
        __Context__: CanWelcome,
    {}
    impl<__Context__, __Components__, __Path__> Welcomer<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<__Path__>>::Delegate: Welcomer<__Context__>,
    {
        fn welcome(__context__: &__Context__) -> String {
            <__Components__ as DelegateComponent<
                __Path__,
            >>::Delegate::welcome(__context__)
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
}
pub mod providers {
    use cgp::prelude::*;
    use crate::components::Welcomer;
    impl<__Context__> Welcomer<__Context__> for WelcomeHello {
        fn welcome(__context__: &__Context__) -> String {
            "Hello".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()>
    for WelcomeHello {}
    ///Provider `WelcomeHello` for [`Welcomer`].
    pub struct WelcomeHello;
}
pub mod contexts {
    pub struct Person;
    impl DelegateComponent<WelcomerComponent> for Person {
        type Delegate = crate::providers::WelcomeHello;
    }
    impl<
        __Context__,
        __Params__: ?Sized,
    > IsProviderFor<WelcomerComponent, __Context__, __Params__> for Person
    where
        crate::providers::WelcomeHello: IsProviderFor<
            WelcomerComponent,
            __Context__,
            __Params__,
        >,
    {}
}
pub mod dependency {
    use cgp::core::error::ErrorTypeProvider;
    use cgp::prelude::*;
    use cgp_test_component_library::direct::Waver;
    use cgp_test_crate_a::Greeter;
    impl<__Context__> ErrorTypeProvider<__Context__> for UseStringError {
        type Error = String;
    }
    impl<__Context__> IsProviderFor<ErrorTypeProviderComponent, __Context__, ()>
    for UseStringError {}
    ///Provider `UseStringError` for [`ErrorTypeProvider`].
    pub struct UseStringError;
    impl<__Context__> Waver<__Context__> for WaveHand {
        fn wave(__context__: &__Context__) -> String {
            "wave".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<WaverComponent, __Context__, ()> for WaveHand {}
    ///Provider `WaveHand` for [`Waver`].
    pub struct WaveHand;
    impl<__Context__> Greeter<__Context__> for GreetFromLibrary {
        fn greet(__context__: &__Context__) -> String {
            "Hello".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()>
    for GreetFromLibrary {}
    ///Provider `GreetFromLibrary` for [`Greeter`].
    pub struct GreetFromLibrary;
}
pub mod attributed_use {
    #[allow(unused_imports)]
    use cgp::prelude::*;
    use crate::components::Welcomer;
    impl<__Context__> Welcomer<__Context__> for WelcomeAgain {
        fn welcome(__context__: &__Context__) -> String {
            "Hello again".to_owned()
        }
    }
    impl<__Context__> IsProviderFor<WelcomerComponent, __Context__, ()>
    for WelcomeAgain {}
    ///Provider `WelcomeAgain` for [`Welcomer`].
    pub struct WelcomeAgain;
}
fn main() {}
