#![feature(prelude_import)]
//! Usability: a provider with two independent unmet field dependencies, which should render as
//! two parallel branches in the dependency tree rather than a single spine.
//!
//! `GreetFullName` needs both `HasFirstName` and `HasLastName`, and `Person` supplies neither
//! field, so the one check failure has two distinct root causes. The dependency note should
//! branch at the provider into a `first_name` path and a `last_name` path.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
pub trait HasFirstName {
    fn first_name(&self) -> &str;
}
impl<__Context__> HasFirstName for __Context__
where
    __Context__: HasField<Symbol!("first_name"), Value = String>,
{
    fn first_name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("first_name")>).as_str()
    }
}
pub trait HasLastName {
    fn last_name(&self) -> &str;
}
impl<__Context__> HasLastName for __Context__
where
    __Context__: HasField<Symbol!("last_name"), Value = String>,
{
    fn last_name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("last_name")>).as_str()
    }
}
impl<__Context__> Greeter<__Context__> for GreetFullName
where
    __Context__: HasFirstName + HasLastName,
{
    fn greet(__context__: &__Context__) {
        {
            ::std::io::_print(
                format_args!(
                    "Hello, {0} {1}!\n", __context__.first_name(), __context__
                    .last_name()
                ),
            );
        };
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetFullName
where
    __Context__: HasFirstName + HasLastName,
{}
pub struct GreetFullName;
pub struct Person {}
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetFullName;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    GreetFullName: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckPerson<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckPerson<GreeterComponent, ()> for Person {}
fn main() {}
