#![feature(prelude_import)]
//! A trait or provider called in impls no CGP macro wrote, where `#[uses]` and `#[use_provider]`
//! cannot go.
//!
//! The impls here sit over a type parameter, as a `#[cgp_fn]` or `#[cgp_impl]` impl does, but they
//! are written by hand, or with `#[cgp_new_provider]`, so the fix is an ordinary bound on the
//! parameter. Each `E0599` keeps rustc's headline, its names resugared, rather than reading as
//! `[CGP-E030]` or `[CGP-E031]`, whose helps name attributes these impls do not take.
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
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
impl<__Context__> CanCalculateArea for __Context__
where
    __Context__: AreaCalculator<__Context__>,
{
    fn area(&self) -> f64 {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
>: IsProviderFor<AreaCalculatorComponent, __Context__, ()> {
    fn area(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> AreaCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
///Component marker for [`CanCalculateArea`].
pub struct AreaCalculatorComponent;
impl<__Context__> AreaCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateArea,
{
    fn area(__context__: &__Context__) -> f64 {
        __Context__::area(__context__)
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for UseContext
where
    __Context__: CanCalculateArea,
{}
impl<__Context__, __Components__, __Path__> AreaCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
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
pub trait Describe {
    fn describe(&self) -> u64;
}
impl<T> Describe for T {
    fn describe(&self) -> u64 {
        self.get_count()
    }
}
pub trait Run<C> {
    fn run(context: &C) -> u64;
}
pub struct Runner;
impl<C> Run<C> for Runner {
    fn run(context: &C) -> u64 {
        context.get_count()
    }
}
pub struct Wrapper<T>(T);
pub trait Twice {
    fn twice(&self) -> f64;
}
impl<T> Twice for Wrapper<T> {
    fn twice(&self) -> f64 {
        T::area(&self.0) * 2.0
    }
}
impl<Context> Greeter<Context> for GreetHello {
    fn greet(context: &Context) -> u64 {
        context.get_count()
    }
}
impl<Context> IsProviderFor<GreeterComponent, Context, ()> for GreetHello {}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
fn main() {}
