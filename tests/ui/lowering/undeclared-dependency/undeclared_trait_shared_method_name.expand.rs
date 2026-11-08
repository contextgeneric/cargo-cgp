#![feature(prelude_import)]
//! A `#[cgp_fn]` body calls `self.describe()` without `#[uses]`, where two CGP traits declare a
//! `describe` method: the `#[cgp_fn]` trait `Describe`, imported, and the consumer trait
//! `CanDescribe` in another module.
//!
//! The tool reports `[CGP-E030]` for `Describe`, the trait rustc's notes point at, rather than the
//! first candidate by name.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub mod named {
    use cgp::prelude::*;
    ///The trait for the `describe` operation.
    pub trait Describe {
        ///The `describe` operation.
        fn describe(&self) -> String;
    }
    impl<__Context__> Describe for __Context__
    where
        Self: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
    {
        fn describe(&self) -> String {
            let name: &str = ::core::convert::AsRef::as_ref(
                HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("name")>),
            );
            name.to_owned()
        }
    }
}
pub mod component {
    use cgp::prelude::*;
    pub trait CanDescribe {
        fn describe(&self) -> String;
    }
    impl<__Context__> CanDescribe for __Context__
    where
        __Context__: Describer<__Context__>,
    {
        fn describe(&self) -> String {
            __Context__::describe(self)
        }
    }
    pub trait Describer<
        __Context__,
    >: IsProviderFor<DescriberComponent, __Context__, ()> {
        fn describe(__context__: &__Context__) -> String;
    }
    impl<__Provider__, __Context__> Describer<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<DescriberComponent>
            + IsProviderFor<DescriberComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            DescriberComponent,
        >>::Delegate: Describer<__Context__>,
    {
        fn describe(__context__: &__Context__) -> String {
            <__Provider__ as DelegateComponent<
                DescriberComponent,
            >>::Delegate::describe(__context__)
        }
    }
    ///Component marker for [`CanDescribe`].
    pub struct DescriberComponent;
    impl<__Context__> Describer<__Context__> for UseContext
    where
        __Context__: CanDescribe,
    {
        fn describe(__context__: &__Context__) -> String {
            __Context__::describe(__context__)
        }
    }
    impl<__Context__> IsProviderFor<DescriberComponent, __Context__, ()> for UseContext
    where
        __Context__: CanDescribe,
    {}
    impl<__Context__, __Components__, __Path__> Describer<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: Describer<__Context__>,
    {
        fn describe(__context__: &__Context__) -> String {
            <__Components__ as DelegateComponent<
                __Path__,
            >>::Delegate::describe(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<DescriberComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<DescriberComponent, __Context__, ()>
            + Describer<__Context__>,
    {}
}
use named::Describe;
///The trait for the `greet` operation.
pub trait Greet {
    ///The `greet` operation.
    fn greet(&self) -> String;
}
impl<__Context__> Greet for __Context__ {
    fn greet(&self) -> String {
        self.describe()
    }
}
fn main() {}
