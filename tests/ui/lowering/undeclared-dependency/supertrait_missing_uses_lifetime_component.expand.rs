#![feature(prelude_import)]
//! A provider for a component with a lifetime parameter and `#[extend(HasName)]` that omits
//! `#[uses(HasName)]`.
//!
//! The component requires `HasName` of every context, so the provider trait carries the bound and
//! rustc reports `E0277` on `__Context__: HasName` at the `#[cgp_impl]` attribute. The provider
//! trait takes the component's lifetime ahead of the context, so a reader that took the context by
//! raw position would land on the lifetime; the tool reads it by type position and reshapes the
//! error into `[CGP-E036]`, as it does for a component without parameters.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
pub trait CanGreet<'a>: HasName {
    fn greet(&self, other: &'a str) -> String;
}
impl<'a, __Context__> CanGreet<'a> for __Context__
where
    __Context__: HasName,
    __Context__: Greeter<'a, __Context__>,
{
    fn greet(&self, other: &'a str) -> String {
        __Context__::greet(self, other)
    }
}
pub trait Greeter<
    'a,
    __Context__,
>: IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String;
}
impl<'a, __Provider__, __Context__> Greeter<'a, __Context__> for __Provider__
where
    __Context__: HasName,
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<'a, __Context__>,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__, other)
    }
}
pub struct GreeterComponent;
impl<'a, __Context__> Greeter<'a, __Context__> for UseContext
where
    __Context__: HasName,
    __Context__: CanGreet<'a>,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        __Context__::greet(__context__, other)
    }
}
impl<'a, __Context__> IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>
for UseContext
where
    __Context__: HasName,
    __Context__: CanGreet<'a>,
{}
impl<'a, __Context__, __Components__, __Path__> Greeter<'a, __Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasName,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<'a, __Context__>,
{
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::greet(__context__, other)
    }
}
impl<
    'a,
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasName,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<'a, __Context__>,
{}
impl<'a, __Context__> Greeter<'a, __Context__> for GreetHello {
    fn greet(__context__: &__Context__, other: &'a str) -> String {
        other.to_owned()
    }
}
impl<'a, __Context__> IsProviderFor<GreeterComponent, __Context__, (Life<'a>)>
for GreetHello {}
pub struct GreetHello;
fn main() {}
