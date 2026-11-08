#![feature(prelude_import)]
//! A consumer trait required by a struct's bound, built with a context whose wiring misses the field
//! the provider reads. The root cause is the missing `name` field on `App`, and `Holder` only passes
//! the requirement on.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
            ::alloc::fmt::format(format_args!("Hello, {0}!", name))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
pub struct Holder<T: CanGreet>(pub T);
pub struct App {
    pub label: String,
}
impl HasField<Symbol!("label")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &Self::Value {
        &self.label
    }
}
impl HasFieldMut<Symbol!("label")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &mut Self::Value {
        &mut self.label
    }
}
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetHello;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    GreetHello: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
fn main() {
    let app = App { label: String::new() };
    let _holder = Holder(app);
}
