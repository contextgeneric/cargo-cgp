#![feature(prelude_import)]
//! Every consumer a use-site failure names gets its chain, sharing one root cause.
//!
//! `App` wires two independent components — `CanGreet` and `CanBidFarewell` — whose
//! providers each read the same `name` field through `HasName`, and `App` does not have
//! that field. Calling either method fails as a use-site `E0599`, and the use-site anchor
//! walks *every* component the context wires, so both fail and both are named in the
//! `[CGP-E001]` header.
//!
//! Pins that both then appear in the note. The anchor unions the two walks' causes, which
//! name one shared leaf, and `merge_causes_by_leaf` folds them into a single cause holding
//! *both* routes — where de-duplicating by leaf and discarding the duplicate's paths would
//! leave the header promising two failing consumers and the note accounting for one. The
//! two chains converge on the shared `HasName` hop, so the second `(*)`-truncates there.
//! The check-entry counterpart of this shape is `parallel_consumers`.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/hidden/unsatisfied-dependency.md
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
impl<__Context__> Greeter<__Context__> for GreetWithName
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", __context__.name()))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetWithName
where
    __Context__: HasName,
{}
pub struct GreetWithName;
pub trait CanBidFarewell {
    fn farewell(&self) -> String;
}
impl<__Context__> CanBidFarewell for __Context__
where
    __Context__: Farewell<__Context__>,
{
    fn farewell(&self) -> String {
        __Context__::farewell(self)
    }
}
pub trait Farewell<__Context__>: IsProviderFor<FarewellComponent, __Context__, ()> {
    fn farewell(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Farewell<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FarewellComponent>
        + IsProviderFor<FarewellComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        FarewellComponent,
    >>::Delegate: Farewell<__Context__>,
{
    fn farewell(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            FarewellComponent,
        >>::Delegate::farewell(__context__)
    }
}
pub struct FarewellComponent;
impl<__Context__> Farewell<__Context__> for UseContext
where
    __Context__: CanBidFarewell,
{
    fn farewell(__context__: &__Context__) -> String {
        __Context__::farewell(__context__)
    }
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for UseContext
where
    __Context__: CanBidFarewell,
{}
impl<__Context__, __Components__, __Path__> Farewell<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Farewell<__Context__>,
{
    fn farewell(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::farewell(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<FarewellComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FarewellComponent, __Context__, ()>
        + Farewell<__Context__>,
{}
impl<__Context__> Farewell<__Context__> for FarewellWithName
where
    __Context__: HasName,
{
    fn farewell(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Goodbye, {0}!", __context__.name()))
        })
    }
}
impl<__Context__> IsProviderFor<FarewellComponent, __Context__, ()> for FarewellWithName
where
    __Context__: HasName,
{}
pub struct FarewellWithName;
pub struct App {
    pub age: u8,
}
impl HasField<Symbol!("age")> for App {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &Self::Value {
        &self.age
    }
}
impl HasFieldMut<Symbol!("age")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &mut Self::Value {
        &mut self.age
    }
}
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = GreetWithName;
}
impl<__Context__, __Params__> IsProviderFor<GreeterComponent, __Context__, __Params__>
for App
where
    GreetWithName: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl DelegateComponent<FarewellComponent> for App {
    type Delegate = FarewellWithName;
}
impl<__Context__, __Params__> IsProviderFor<FarewellComponent, __Context__, __Params__>
for App
where
    FarewellWithName: IsProviderFor<FarewellComponent, __Context__, __Params__>,
{}
fn main() {
    let app = App { age: 8 };
    let _ = app.greet();
}
