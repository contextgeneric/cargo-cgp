#![feature(prelude_import)]
//! Acceptable failure: a child namespace that inherits a parent and then
//! *redefines* a key the parent already binds — a namespace entry cannot be
//! overridden by an inheriting namespace.
//!
//! `new ChildNs: BaseNs` emits the inheritance blanket impl `impl<Table, Key,
//! Value> ChildNs<Table> for Key where Key: BaseNs<__ChildNsComponents>, Key:
//! BaseNs<Table, Delegate = Value>`, which forwards *every* key `BaseNs` resolves —
//! including `GreeterComponent`, since `BaseNs` binds it. The child's own
//! `GreeterComponent: GreetBye` entry emits a second impl `impl<Table> ChildNs<Table>
//! for GreeterComponent`, and the two overlap for that key, so coherence rejects the
//! pair (`E0119`, a *single* conflict on `ChildNs<_> for GreeterComponent`, since a
//! namespace emits only its own lookup-trait impl, not the context-side
//! `DelegateComponent`/`IsProviderFor` pair). Inheritance layers new keys onto a
//! parent; it cannot revise the parent's existing keys. To vary a key per
//! configuration, leave it *unbound* in the shared base and bind it in each child,
//! rather than binding it in the base and overriding it. CGP lowers both impls
//! faithfully; only the whole program reveals the overlap, so it defers to the
//! compiler.
//!
//! This is the namespace-level (inheritance) shape of the override-conflict class;
//! contrast the context-level shape in override_registered_path.rs, where a context
//! joining a namespace tries to override a path the namespace registers.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md.
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
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "Hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
impl<__Context__> Greeter<__Context__> for GreetBye {
    fn greet(__context__: &__Context__) -> String {
        "Bye".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetBye {}
pub struct GreetBye;
pub struct __BaseNsComponents;
pub trait BaseNs<__Table__> {
    type Delegate;
}
impl<__Table__> BaseNs<__Table__> for GreeterComponent {
    type Delegate = GreetHello;
}
pub struct __ChildNsComponents;
pub trait ChildNs<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> ChildNs<__Table__> for __Key__
where
    __Key__: BaseNs<__ChildNsComponents>,
    __Key__: BaseNs<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<__Table__> ChildNs<__Table__> for GreeterComponent {
    type Delegate = GreetBye;
}
fn main() {}
