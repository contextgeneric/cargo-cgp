#![feature(prelude_import)]
//! One root cause reported once across a chain of dependent components.
//!
//! `ProvideFoo` needs the `name` field, `ProvideBar` depends on `CanFoo`, and
//! `ProvideBaz` depends on `CanBar`; wiring all three onto an `App` without a `name`
//! field and checking all three would, left to rustc, cascade into a block per
//! component — more than three, since the deeper providers also emit intermediate
//! provider-bound failures. cargo-cgp coalesces them: the three consumer failures
//! share the one missing-`name` root cause, so they collapse into a single
//! `[CGP-E001]` headline naming `CanBaz`, `CanBar`, and `CanFoo`, a caret at each check
//! entry, and one representative dependency chain down to the missing field (`CanBaz`,
//! whose chain is the deepest and subsumes the others, chosen regardless of check
//! order). Fixing
//! the one field clears the whole cascade. One entry surfaces to rustc as a
//! provider-side bound, but coalescing words the group uniformly as consumer traits,
//! since a `check_components!` entry failing *is* the consumer trait failing.
//!
//! See docs/errors/checks/verbose-cascade.md.
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
pub trait CanFoo {
    fn foo(&self);
}
impl<__Context__> CanFoo for __Context__
where
    __Context__: Foo<__Context__>,
{
    fn foo(&self) {
        __Context__::foo(self)
    }
}
pub trait Foo<__Context__>: IsProviderFor<FooComponent, __Context__, ()> {
    fn foo(__context__: &__Context__);
}
impl<__Provider__, __Context__> Foo<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<FooComponent>
        + IsProviderFor<FooComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<FooComponent>>::Delegate: Foo<__Context__>,
{
    fn foo(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<FooComponent>>::Delegate::foo(__context__)
    }
}
pub struct FooComponent;
impl<__Context__> Foo<__Context__> for UseContext
where
    __Context__: CanFoo,
{
    fn foo(__context__: &__Context__) {
        __Context__::foo(__context__)
    }
}
impl<__Context__> IsProviderFor<FooComponent, __Context__, ()> for UseContext
where
    __Context__: CanFoo,
{}
impl<__Context__, __Components__, __Path__> Foo<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Foo<__Context__>,
{
    fn foo(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::foo(__context__)
    }
}
impl<__Context__, __Components__, __Path__> IsProviderFor<FooComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<FooComponent, __Context__, ()> + Foo<__Context__>,
{}
pub trait CanBar {
    fn bar(&self);
}
impl<__Context__> CanBar for __Context__
where
    __Context__: Bar<__Context__>,
{
    fn bar(&self) {
        __Context__::bar(self)
    }
}
pub trait Bar<__Context__>: IsProviderFor<BarComponent, __Context__, ()> {
    fn bar(__context__: &__Context__);
}
impl<__Provider__, __Context__> Bar<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<BarComponent>
        + IsProviderFor<BarComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<BarComponent>>::Delegate: Bar<__Context__>,
{
    fn bar(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<BarComponent>>::Delegate::bar(__context__)
    }
}
pub struct BarComponent;
impl<__Context__> Bar<__Context__> for UseContext
where
    __Context__: CanBar,
{
    fn bar(__context__: &__Context__) {
        __Context__::bar(__context__)
    }
}
impl<__Context__> IsProviderFor<BarComponent, __Context__, ()> for UseContext
where
    __Context__: CanBar,
{}
impl<__Context__, __Components__, __Path__> Bar<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Bar<__Context__>,
{
    fn bar(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::bar(__context__)
    }
}
impl<__Context__, __Components__, __Path__> IsProviderFor<BarComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<BarComponent, __Context__, ()> + Bar<__Context__>,
{}
pub trait CanBaz {
    fn baz(&self);
}
impl<__Context__> CanBaz for __Context__
where
    __Context__: Baz<__Context__>,
{
    fn baz(&self) {
        __Context__::baz(self)
    }
}
pub trait Baz<__Context__>: IsProviderFor<BazComponent, __Context__, ()> {
    fn baz(__context__: &__Context__);
}
impl<__Provider__, __Context__> Baz<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<BazComponent>
        + IsProviderFor<BazComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<BazComponent>>::Delegate: Baz<__Context__>,
{
    fn baz(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<BazComponent>>::Delegate::baz(__context__)
    }
}
pub struct BazComponent;
impl<__Context__> Baz<__Context__> for UseContext
where
    __Context__: CanBaz,
{
    fn baz(__context__: &__Context__) {
        __Context__::baz(__context__)
    }
}
impl<__Context__> IsProviderFor<BazComponent, __Context__, ()> for UseContext
where
    __Context__: CanBaz,
{}
impl<__Context__, __Components__, __Path__> Baz<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Baz<__Context__>,
{
    fn baz(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::baz(__context__)
    }
}
impl<__Context__, __Components__, __Path__> IsProviderFor<BazComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<BazComponent, __Context__, ()> + Baz<__Context__>,
{}
impl<__Context__> Foo<__Context__> for ProvideFoo
where
    __Context__: HasName,
{
    fn foo(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__> IsProviderFor<FooComponent, __Context__, ()> for ProvideFoo
where
    __Context__: HasName,
{}
pub struct ProvideFoo;
impl<__Context__> Bar<__Context__> for ProvideBar
where
    __Context__: CanFoo,
{
    fn bar(__context__: &__Context__) {
        __context__.foo();
    }
}
impl<__Context__> IsProviderFor<BarComponent, __Context__, ()> for ProvideBar
where
    __Context__: CanFoo,
{}
pub struct ProvideBar;
impl<__Context__> Baz<__Context__> for ProvideBaz
where
    __Context__: CanBar,
{
    fn baz(__context__: &__Context__) {
        __context__.bar();
    }
}
impl<__Context__> IsProviderFor<BazComponent, __Context__, ()> for ProvideBaz
where
    __Context__: CanBar,
{}
pub struct ProvideBaz;
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
impl DelegateComponent<FooComponent> for App {
    type Delegate = ProvideFoo;
}
impl<__Context__, __Params__> IsProviderFor<FooComponent, __Context__, __Params__>
for App
where
    ProvideFoo: IsProviderFor<FooComponent, __Context__, __Params__>,
{}
impl DelegateComponent<BarComponent> for App {
    type Delegate = ProvideBar;
}
impl<__Context__, __Params__> IsProviderFor<BarComponent, __Context__, __Params__>
for App
where
    ProvideBar: IsProviderFor<BarComponent, __Context__, __Params__>,
{}
impl DelegateComponent<BazComponent> for App {
    type Delegate = ProvideBaz;
}
impl<__Context__, __Params__> IsProviderFor<BazComponent, __Context__, __Params__>
for App
where
    ProvideBaz: IsProviderFor<BazComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<BazComponent, ()> for App {}
impl __CheckApp<BarComponent, ()> for App {}
impl __CheckApp<FooComponent, ()> for App {}
fn main() {}
