#![feature(prelude_import)]
//! A cross-context dependency — one context's wiring depending on a *concrete* other context —
//! resolved cleanly, and the shape that makes the resolution cache's per-context node key
//! load-bearing. `Inner` wires and checks its own `CanCompute` component, whose provider needs a
//! `name` field `Inner` lacks. `Outer` wires a `CanRun` provider whose `where Inner: CanCompute`
//! clause depends on `Inner`, so the obligation `Inner: CanCompute` appears in both dependency trees
//! — as `Inner`'s own checked consumer (root context `Inner`) and as an interior node of `Outer`'s
//! tree (root context `Outer`).
//!
//! Three behaviors combine here:
//!   - The provider impl's own `where Inner: CanCompute` clause is recovered as the consumer
//!     obligation it is (the impl-site anchor reads the concrete-context bound directly rather than
//!     declining on the provider impl), so it de-duplicates into `Inner`'s own `[CGP-E001]` block
//!     rather than leaving rustc's raw bound error — the two sites of one mistake collapse to one.
//!   - `Outer`'s tree re-roots the `Inner: CanCompute` node at `Inner` while walking, so it decodes
//!     to `[CGP-E106] missing field \`name\`` (not an opaque bound), its delegation-routing hop is
//!     dropped, and it reads as a consumer node `for context Inner`.
//!   - The node renders as a `[CGP-E101] consumer trait impl … for context \`Inner\`` in both trees
//!     precisely because the cache keys each node on `(obligation, context)`: `Outer`'s walk re-roots
//!     to `Inner` and thereby shares `Inner`'s own cached subtree, rather than borrowing `Outer`'s
//!     context. Were the key the obligation alone, one tree would splice in the other's labels.
//!
//! See docs/implementation/cached-dependency-resolution.md (the cache key) and
//! docs/implementation/typed-root-cause-resolution.md (the impl-site anchor and cross-context walk).
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
pub trait CanCompute {
    fn compute(&self);
}
impl<__Context__> CanCompute for __Context__
where
    __Context__: Computer<__Context__>,
{
    fn compute(&self) {
        __Context__::compute(self)
    }
}
pub trait Computer<__Context__>: IsProviderFor<ComputerComponent, __Context__, ()> {
    fn compute(__context__: &__Context__);
}
impl<__Provider__, __Context__> Computer<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ComputerComponent>
        + IsProviderFor<ComputerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ComputerComponent,
    >>::Delegate: Computer<__Context__>,
{
    fn compute(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<
            ComputerComponent,
        >>::Delegate::compute(__context__)
    }
}
pub struct ComputerComponent;
impl<__Context__> Computer<__Context__> for UseContext
where
    __Context__: CanCompute,
{
    fn compute(__context__: &__Context__) {
        __Context__::compute(__context__)
    }
}
impl<__Context__> IsProviderFor<ComputerComponent, __Context__, ()> for UseContext
where
    __Context__: CanCompute,
{}
impl<__Context__, __Components__, __Path__> Computer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Computer<__Context__>,
{
    fn compute(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::compute(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ComputerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ComputerComponent, __Context__, ()>
        + Computer<__Context__>,
{}
impl<__Context__> Computer<__Context__> for DoCompute
where
    __Context__: HasName,
{
    fn compute(__context__: &__Context__) {
        let _ = __context__.name();
    }
}
impl<__Context__> IsProviderFor<ComputerComponent, __Context__, ()> for DoCompute
where
    __Context__: HasName,
{}
pub struct DoCompute;
pub trait CanRun {
    fn run(&self);
}
impl<__Context__> CanRun for __Context__
where
    __Context__: Runner<__Context__>,
{
    fn run(&self) {
        __Context__::run(self)
    }
}
pub trait Runner<__Context__>: IsProviderFor<RunnerComponent, __Context__, ()> {
    fn run(__context__: &__Context__);
}
impl<__Provider__, __Context__> Runner<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<RunnerComponent>
        + IsProviderFor<RunnerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<RunnerComponent>>::Delegate: Runner<__Context__>,
{
    fn run(__context__: &__Context__) {
        <__Provider__ as DelegateComponent<RunnerComponent>>::Delegate::run(__context__)
    }
}
pub struct RunnerComponent;
impl<__Context__> Runner<__Context__> for UseContext
where
    __Context__: CanRun,
{
    fn run(__context__: &__Context__) {
        __Context__::run(__context__)
    }
}
impl<__Context__> IsProviderFor<RunnerComponent, __Context__, ()> for UseContext
where
    __Context__: CanRun,
{}
impl<__Context__, __Components__, __Path__> Runner<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Runner<__Context__>,
{
    fn run(__context__: &__Context__) {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::run(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<RunnerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<RunnerComponent, __Context__, ()> + Runner<__Context__>,
{}
impl<__Context__> Runner<__Context__> for RunViaInner
where
    Inner: CanCompute,
{
    fn run(__context__: &__Context__) {}
}
impl<__Context__> IsProviderFor<RunnerComponent, __Context__, ()> for RunViaInner
where
    Inner: CanCompute,
{}
pub struct RunViaInner;
pub struct Inner {
    pub age: u8,
}
impl HasField<Symbol!("age")> for Inner {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &Self::Value {
        &self.age
    }
}
impl HasFieldMut<Symbol!("age")> for Inner {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("age")>,
    ) -> &mut Self::Value {
        &mut self.age
    }
}
pub struct Outer {
    pub label: u8,
}
impl HasField<Symbol!("label")> for Outer {
    type Value = u8;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &Self::Value {
        &self.label
    }
}
impl HasFieldMut<Symbol!("label")> for Outer {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("label")>,
    ) -> &mut Self::Value {
        &mut self.label
    }
}
impl DelegateComponent<ComputerComponent> for Inner {
    type Delegate = DoCompute;
}
impl<__Context__, __Params__> IsProviderFor<ComputerComponent, __Context__, __Params__>
for Inner
where
    DoCompute: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<RunnerComponent> for Outer {
    type Delegate = RunViaInner;
}
impl<__Context__, __Params__> IsProviderFor<RunnerComponent, __Context__, __Params__>
for Outer
where
    RunViaInner: IsProviderFor<RunnerComponent, __Context__, __Params__>,
{}
trait __CheckInner<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckInner<ComputerComponent, ()> for Inner {}
trait __CheckOuter<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckOuter<RunnerComponent, ()> for Outer {}
fn main() {}
