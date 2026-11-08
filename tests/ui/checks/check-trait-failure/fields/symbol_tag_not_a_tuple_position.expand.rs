#![feature(prelude_import)]
//! `Symbol!` tags on tuple structs that spell no position of the struct.
//!
//! `GreetThird` requires the field `Symbol!("2")` of `Pair`, which has two fields, and
//! `GreetPadded` requires `Symbol!("01")` of `Couple`, a padded spelling of a position. A tuple
//! struct keys its fields by `Index<N>`, so each check reports a missing field, and neither gets the
//! `help` naming an `Index` tag, since no field sits at position 2 and `01` is no position's name.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGreet {
    fn greet(&self) -> u32;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> u32 {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> u32;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> u32 {
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
    fn greet(__context__: &__Context__) -> u32 {
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
    fn greet(__context__: &__Context__) -> u32 {
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
impl<__Context__> Greeter<__Context__> for GreetThird
where
    __Context__: HasField<Symbol!("2"), Value = u32>,
{
    fn greet(__context__: &__Context__) -> u32 {
        *__context__.get_field(PhantomData::<Symbol!("2")>)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetThird
where
    __Context__: HasField<Symbol!("2"), Value = u32>,
{}
///Provider `GreetThird` for [`Greeter`].
pub struct GreetThird;
impl<__Context__> Greeter<__Context__> for GreetPadded
where
    __Context__: HasField<Symbol!("01"), Value = u32>,
{
    fn greet(__context__: &__Context__) -> u32 {
        *__context__.get_field(PhantomData::<Symbol!("01")>)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetPadded
where
    __Context__: HasField<Symbol!("01"), Value = u32>,
{}
///Provider `GreetPadded` for [`Greeter`].
pub struct GreetPadded;
pub struct Pair(pub u32, pub u32);
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
    type Value = u32;
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
pub struct Couple(pub u32, pub u32);
impl HasField<Index<0>> for Couple {
    type Value = u32;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<0>>) -> &Self::Value {
        &self.0
    }
}
impl HasFieldMut<Index<0>> for Couple {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<0>>,
    ) -> &mut Self::Value {
        &mut self.0
    }
}
impl HasField<Index<1>> for Couple {
    type Value = u32;
    fn get_field(&self, key: ::core::marker::PhantomData<Index<1>>) -> &Self::Value {
        &self.1
    }
}
impl HasFieldMut<Index<1>> for Couple {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Index<1>>,
    ) -> &mut Self::Value {
        &mut self.1
    }
}
impl DelegateComponent<GreeterComponent> for Pair {
    type Delegate = GreetThird;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Pair
where
    GreetThird: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<GreeterComponent> for Couple {
    type Delegate = GreetPadded;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for Couple
where
    GreetPadded: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
trait __CheckPair<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckPair<GreeterComponent, ()> for Pair {}
trait CheckCouple<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl CheckCouple<GreeterComponent, ()> for Couple {}
fn main() {}
