#![feature(prelude_import)]
//! A plain Rust error inside a hand-written impl whose CGP supertrait also fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `CanGreetTwice`'s supertrait
//! `CanGreet` fails. The impl body also passes an `Rc` to a `Send` bound, a separate mistake. The
//! tool reports both, as plain `cargo check` does: the `[CGP-E003]` block on the impl, and the
//! `Send` error as rustc wrote it. The impl-site anchor takes a caret inside a method body only on a
//! call to a method of the impl's trait or its supertraits, so the forwarding `self.greet()` error
//! joins the CGP block while the `Send` error keeps its own.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::rc::Rc;
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
impl<__Context__> Greeter<__Context__> for GreetHello
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
            ::alloc::fmt::format(format_args!("Hello, {0}", name))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{}
///Provider `GreetHello` for [`Greeter`].
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
fn needs_send<T: Send>(_value: T) {}
impl CanGreetTwice for Person {
    fn greet_twice(&self) -> String {
        needs_send(Rc::new(1));
        self.greet()
    }
}
fn main() {}
