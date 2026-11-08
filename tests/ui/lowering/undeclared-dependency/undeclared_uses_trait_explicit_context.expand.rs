#![feature(prelude_import)]
//! A `#[cgp_impl]` provider in the explicit-context form calling a trait it does not declare.
//!
//! `impl<Context> Greeter for Context` names its own context, so the impl declares no reserved
//! `__Context__`, but `#[cgp_impl]` lowers it all the same and takes `#[uses]`. It is the macro's
//! own impl, which `undeclared_calls_in_plain_impls` tells apart from a hand-written one.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
///The trait for the `get_count` operation.
pub trait GetCount {
    ///The `get_count` operation.
    fn get_count(&self) -> u64;
}
impl<__Context__> GetCount for __Context__
where
    Self: HasField<Symbol!("count"), Value = u64>,
{
    fn get_count(&self) -> u64 {
        let count: &u64 = HasField::get_field(
            self,
            ::core::marker::PhantomData::<Symbol!("count")>,
        );
        *count
    }
}
pub trait CanGreet {
    fn greet(&self) -> u64;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> u64 {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> u64;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> u64 {
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
    fn greet(__context__: &__Context__) -> u64 {
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
    fn greet(__context__: &__Context__) -> u64 {
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
impl<Context> Greeter<Context> for GreetExplicit {
    fn greet(__context__: &Context) -> u64 {
        __context__.get_count()
    }
}
impl<Context> IsProviderFor<GreeterComponent, Context, ()> for GreetExplicit {}
///Provider `GreetExplicit` for [`Greeter`].
pub struct GreetExplicit;
fn main() {}
