#![feature(prelude_import)]
//! Usability: a CGP check failure and an ordinary Rust error in the same crate, to confirm the
//! typed resolver replaces only the CGP wiring diagnostic and leaves the plain error untouched.
//!
//! `Person` is missing the `name` field its `GreetHello` provider needs, which the resolver
//! turns into a dependency tree. The `wrong_type` function has an unrelated `E0308` type
//! mismatch, which is not a check-entry `E0277`, so the resolver declines it and it flows
//! through the fallback pipeline unchanged. Both diagnostics should appear.
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
impl<__Context__> Greeter<__Context__> for GreetHello
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) {
        {
            ::std::io::_print(format_args!("Hello, {0}!\n", __context__.name()));
        };
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasName,
{}
pub struct GreetHello;
pub struct Person {}
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHello;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for Person
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckPerson<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckPerson<GreeterComponent, ()> for Person {}
fn wrong_type() -> u32 {
    "not a number"
}
fn main() {}
