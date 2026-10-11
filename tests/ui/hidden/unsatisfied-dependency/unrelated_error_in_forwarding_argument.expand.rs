#![feature(prelude_import)]
//! A plain Rust error inside an argument of a hand-written impl's forwarding call, whose CGP
//! supertrait also fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `CanGreetTwice`'s supertrait
//! `CanGreet` fails. The forwarding `self.greet(...)` passes an `Rc` to a `Send` bound in its
//! argument, a separate mistake inside the call. Plain `cargo check` reports both; the tool reports
//! only the `[CGP-E009]` block on the impl, and the `Send` error is lost.
//!
//! This is the impl-site form of the call-site limitation in `unrelated_error_in_call_argument`:
//! the impl-site anchor takes a caret inside a method body on a call to a method of the impl's
//! trait or its supertraits, and the `Send` error's caret lies inside such a call's argument.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::rc::Rc;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self, times: u32) -> String;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self, times: u32) -> String {
        __Context__::greet(self, times)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__, times: u32) -> String;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__, times: u32) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__, times)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__, times: u32) -> String {
        __Context__::greet(__context__, times)
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
    fn greet(__context__: &__Context__, times: u32) -> String {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::greet(__context__, times)
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
impl<__Context__> Greeter<__Context__> for GreetHello
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn greet(__context__: &__Context__, times: u32) -> String {
        let name: &str = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0} x{1}", name, times))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{}
pub struct GreetHello;
pub struct Person {
    pub age: u8,
}
impl HasField<Symbol!("age")> for Person {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &Self::Value {
        &self.age
    }
}
impl HasFieldMut<Symbol!("age")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &mut Self::Value {
        &mut self.age
    }
}
impl DelegateComponent<GreeterComponent> for Person {
    type Delegate = GreetHello;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Person
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
pub trait CanGreetTwice: CanGreet {
    fn greet_twice(&self) -> String;
}
fn count<T: Send>(_value: T) -> u32 {
    1
}
impl CanGreetTwice for Person {
    fn greet_twice(&self) -> String {
        self.greet(count(Rc::new(1)))
    }
}
fn main() {}
