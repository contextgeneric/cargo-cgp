#![feature(prelude_import)]
//! One underived field named once, however many coalesced consumers reach it.
//!
//! `App` declares a `name` field but no `#[derive(HasField)]`, so the field is
//! present-but-underived — one mistake with one fix. Three checked components read it
//! through a cascade (`ProvideBaz` needs `CanBar`, which needs `CanFoo`, which reads the
//! field), so all three consumer failures share that single root cause and coalesce into
//! one `[CGP-E001]` block.
//!
//! Pins that the merged block states the shared cause *once*. Coalescing several
//! **distinct** underived fields on one struct into one lead is deliberate — the derive
//! emits an impl per field, so they are one fix (`base_area_2`) — but every member here
//! contributes the **same** field, and the union of their causes therefore repeats it once
//! per member. `merge_causes_by_leaf` folds those copies back into one cause holding all
//! three paths before the underived-field coalescing runs, so the lead keeps its
//! single-field wording rather than reading "the fields `name`, `name`, and `name`", and
//! the merged tree still renders as the one subsuming chain. The cascade's missing-field
//! sibling, where the leaf never coalesces, is `dependency_cascade`.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md
//! (derive-missing variant).
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
    pub name: String,
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
