#![feature(prelude_import)]
//! A `#[cgp_impl]` provider body that calls a trait method without declaring the trait via
//! `#[uses]`: the provider counterpart of [`undeclared_uses_trait`](undeclared_uses_trait.rs).
//!
//! `#[cgp_impl]` lowers the provider into `impl<__Context__> Greeter<__Context__> for GreetHello`,
//! renaming `self` to a `__context__: &__Context__` parameter. A trait the body calls on `self` must
//! be a `where` bound on `__Context__`, declared with `#[uses(…)]`. Here the body calls
//! `self.person_name()`, a `#[cgp_fn]` trait, and declares nothing, so the call cannot resolve.
//!
//! Raw rustc reports an `E0599` saying the method exists for `&__Context__` but its trait bounds
//! were not satisfied, naming the generated `__Context__` and pointing at a `HasField` bound of the
//! called trait rather than at the missing `#[uses(PersonName)]`. Unlike the `#[cgp_fn]` case, the
//! generated impl's `Self` is the provider struct, not the bare `__Context__` parameter, so the
//! failing call's receiver, a parameter declared as `&__Context__`, is what identifies the shape.
//!
//! The tool reshapes it into the same `[CGP-E012]` error as the `#[cgp_fn]` case, with a
//! `help: declare it as a dependency with `#[uses(PersonName)]``, recovered by
//! `resolve::detect_undeclared_trait` from the receiver's declared type. CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
trait PersonName {
    fn person_name(&self) -> String;
}
impl<__Context__> PersonName for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn person_name(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
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
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", __context__.person_name()))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
fn main() {}
