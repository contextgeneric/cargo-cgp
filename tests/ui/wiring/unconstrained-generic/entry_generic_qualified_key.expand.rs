#![feature(prelude_import)]
//! A key's qualified path segment reads by its own name in `[CGP-E019]`.
//!
//! The entry's key is written `std::io::Error`, and the message says `Error`. An `E0207` carries no
//! type in its message, so the detector prints the key's type by its own name, which avoids the
//! first trimmed-path print and the diagnostic lock that print takes. Beside a local `Error`, as
//! here, the name alone does not say which type the entry means, and the caret on the entry does.
//! The other wiring-key codes on such errors, `[CGP-E017]`, `[CGP-E018]`, and `[CGP-E022]`, print the
//! same way.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md>.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Error;
#[automatically_derived]
impl ::core::fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Error")
    }
}
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
pub struct GreetWith<T>(pub T);
pub struct App;
impl<
    T,
    __Wildcard__,
> DelegateComponent<PathCons<GreeterComponent, PathCons<std::io::Error, __Wildcard__>>>
for App {
    type Delegate = GreetWith<T>;
}
impl<
    T,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<GreeterComponent, PathCons<std::io::Error, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetWith<
        T,
    >: IsProviderFor<
        PathCons<GreeterComponent, PathCons<std::io::Error, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<GreeterComponent, PathCons<Error, __Wildcard__>>> for App {
    type Delegate = GreetWith<u8>;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<GreeterComponent, PathCons<Error, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetWith<
        u8,
    >: IsProviderFor<
        PathCons<GreeterComponent, PathCons<Error, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
