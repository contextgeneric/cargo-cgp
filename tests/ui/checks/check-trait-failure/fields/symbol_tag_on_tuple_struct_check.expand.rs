#![feature(prelude_import)]
//! A provider that requires a tuple field by `Symbol!("0")`, checked on a tuple struct.
//!
//! `Pair` derives `HasField`, keyed by `Index<N>`, so `HasField<Symbol!("0")>` does not hold: a
//! digit string names no field. The block states the field is missing, and its `help` names the
//! `Index<0>` tag the field takes. A tuple struct's positions are no field names, so the block does
//! not take the tag for a field `Pair` carries without the derive, and does not blame the derive,
//! which `Pair` has.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
impl<__Context__> Greeter<__Context__> for GreetFirst
where
    __Context__: HasField<Symbol!("0"), Value = u32>,
{
    fn greet(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(
                format_args!(
                    "hello {0}", __context__
                    .get_field(PhantomData::<::cgp::macro_prelude::Symbol < 1,
                    ::cgp::macro_prelude::Chars <'0', ::cgp::macro_prelude::Nil >>>)
                ),
            )
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetFirst
where
    __Context__: HasField<Symbol!("0"), Value = u32>,
{}
pub struct GreetFirst;
pub struct Pair(pub u32, pub String);
impl HasField<Index<0>> for Pair {
    type Value = u32;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
impl HasFieldMut<Index<0>> for Pair {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<0>>,
    ) -> &mut Self::Value {
        &mut self.0
    }
}
impl HasField<Index<1>> for Pair {
    type Value = String;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<1>>) -> &Self::Value {
        &self.1
    }
}
impl HasFieldMut<Index<1>> for Pair {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<1>>,
    ) -> &mut Self::Value {
        &mut self.1
    }
}
impl DelegateComponent<GreeterComponent> for Pair {
    type Delegate = GreetFirst;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Pair
where
    GreetFirst: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckPair<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckPair<GreeterComponent, ()> for Pair {}
fn main() {}
