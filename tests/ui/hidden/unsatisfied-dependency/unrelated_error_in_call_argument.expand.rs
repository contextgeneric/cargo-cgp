#![feature(prelude_import)]
//! A plain Rust error inside an argument of a consumer method call whose wiring fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `person.greet(...)` fails. Its
//! argument passes an `Rc` to a `Send` bound, a separate mistake inside the call. Plain
//! `cargo check` reports both; the tool reports only the `[CGP-E001]` block, and the `Send` error
//! is lost.
//!
//! This is a known limitation of the call-site anchor. It matches a call by the diagnostic's
//! primary span, and the `Send` error's span lies inside the call's argument, so the anchor reads it
//! as another report of the failing call, resolves it to the same cause, and de-duplicates it away.
//! A CGP failure can also point into an argument (the `PhantomData` tag of a handler call), and
//! rustc's `E0277` carries no structured predicate to tell the two apart.
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
fn count<T: Send>(_value: T) -> u32 {
    1
}
fn run(person: &Person) -> String {
    person.greet(count(Rc::new(1)))
}
fn main() {
    let _ = run(&Person { age: 3 });
}
