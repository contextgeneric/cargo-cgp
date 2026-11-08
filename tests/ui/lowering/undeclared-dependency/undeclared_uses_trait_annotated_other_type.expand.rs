#![feature(prelude_import)]
//! A `#[cgp_impl]` provider body that calls a trait through a `let` annotated with a type parameter
//! other than the context.
//!
//! `#[uses]` only adds bounds on the context, so declaring the trait would not fix this: the provider
//! needs a bound on `Value`. rustc's `E0599` names `Value`, and the undeclared-trait detector leaves
//! it alone.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
///The trait for the `person_name` operation.
trait PersonName {
    ///The `person_name` operation.
    fn person_name(&self) -> String;
}
impl<__Context__> PersonName for __Context__
where
    Self: HasField<Symbol!("name"), Value: ::core::convert::AsRef<str> + 'static>,
{
    fn person_name(&self) -> String {
        let name: &str = ::core::convert::AsRef::as_ref(
            HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("name")>),
        );
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
impl<__Context__, Value: Default> Greeter<__Context__> for GreetValue<Value> {
    fn greet(__context__: &__Context__) -> String {
        let made = Value::default();
        let value: &Value = &made;
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", value.person_name()))
        })
    }
}
impl<__Context__, Value: Default> IsProviderFor<GreeterComponent, __Context__, ()>
for GreetValue<Value> {}
///Provider `GreetValue` for [`Greeter`].
pub struct GreetValue<Value>(pub ::core::marker::PhantomData<Value>);
fn main() {}
