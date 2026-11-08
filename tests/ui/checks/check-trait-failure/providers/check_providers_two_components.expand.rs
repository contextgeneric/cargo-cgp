#![feature(prelude_import)]
//! `#[check_providers(GreetAndBye)]` over two components that each fail for a field of their own.
//!
//! The table generates one check impl per component, both with `GreetAndBye` as `Self` on the one
//! token the attribute names, so the two errors share a caret. The tool reads each error's own
//! impl and gives one `[CGP-E002]` block per component: `Greeter` needs `name`, `Farewell` needs
//! `title`. The caret cannot tell the impls apart, because they differ only in their spans' syntax
//! contexts, which the compiler reuses when it does not compile incrementally, so the anchor reads
//! the component each error's own message names.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
pub trait CanSayBye {
    fn bye(&self) -> String;
}
impl<__Context__> CanSayBye for __Context__
where
    __Context__: Farewell<__Context__>,
{
    fn bye(&self) -> String {
        __Context__::bye(self)
    }
}
pub trait Farewell<__Context__>: IsProviderFor<FarewellComponent, __Context__, ()> {
    fn bye(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Farewell<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FarewellComponent>
        + IsProviderFor<FarewellComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FarewellComponent,
    >>::Delegate: Farewell<__Context__>,
{
    fn bye(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            FarewellComponent,
        >>::Delegate::bye(__context__)
    }
}
///Component marker for [`CanSayBye`].
pub struct FarewellComponent;
impl<__Context__> Farewell<__Context__> for UseContext
where
    __Context__: CanSayBye,
{
    fn bye(__context__: &__Context__) -> String {
        __Context__::bye(__context__)
    }
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for UseContext
where
    __Context__: CanSayBye,
{}
impl<__Context__, __Components__, __Path__> Farewell<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Farewell<__Context__>,
{
    fn bye(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::bye(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FarewellComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FarewellComponent, __Context__, ()>
        + Farewell<__Context__>,
{}
impl<__Context__> Greeter<__Context__> for GreetAndBye
where
    __Context__: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{
    fn greet(__context__: &__Context__) -> String {
        let name: &str = ::core::convert::AsRef::as_ref(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("name")>,
            ),
        );
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("hello {0}", name))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetAndBye
where
    __Context__: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{}
///Provider `GreetAndBye` for [`Greeter`].
pub struct GreetAndBye;
impl<__Context__> Farewell<__Context__> for GreetAndBye
where
    __Context__: HasField<
        Symbol!("title"),
        Value: ::core::convert::AsRef<str> + 'static,
    >,
{
    fn bye(__context__: &__Context__) -> String {
        let title: &str = ::core::convert::AsRef::as_ref(
            HasField::get_field(
                __context__,
                ::core::marker::PhantomData::<Symbol!("title")>,
            ),
        );
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("bye {0}", title))
        })
    }
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for GreetAndBye
where
    __Context__: HasField<
        Symbol!("title"),
        Value: ::core::convert::AsRef<str> + 'static,
    >,
{}
pub struct App;
trait __CheckApp<
    __Component__,
    __Context__,
    __Params__: ?Sized,
>: IsProviderFor<__Component__, __Context__, __Params__> {}
impl __CheckApp<GreeterComponent, App, ()> for GreetAndBye {}
impl __CheckApp<FarewellComponent, App, ()> for GreetAndBye {}
fn main() {}
