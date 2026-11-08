#![feature(prelude_import)]
//! A provider's `#[default_impl]` registers a bare component key into a namespace written after
//! it, whose parent already routes that component under its `#[prefix]` path. The registration and
//! the namespace's inheritance blanket conflict with the coherence error E0119, and rustc names the
//! registration as the first implementation and the blanket as the conflicting one.
//!
//! The tool reads the blanket's redirect for the registered key as it does when the namespace comes
//! first ([namespace_inherited_unprefixed_key.rs](namespace_inherited_unprefixed_key.rs)):
//! `[CGP-E013] conflicting wiring: `AppNamespace` wires component `GreeterComponent` directly, but
//! it is redirected to `@app.GreeterComponent``, with the help naming the path key.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E013).
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
impl<__Components__> DefaultNamespace<__Components__> for GreeterComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.GreeterComponent)>;
}
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "Hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
///Provider `GreetHello` for [`Greeter`].
pub struct GreetHello;
impl<__Components__> AppNamespace<__Components__> for GreeterComponent {
    type Delegate = GreetHello;
}
///Wiring table for the `AppNamespace` namespace.
pub struct __AppNamespaceComponents;
///The `AppNamespace` namespace's wiring table.
pub trait AppNamespace<__Table__> {
    ///The provider table the `AppNamespace` namespace delegates to.
    type Delegate;
}
impl<__Table__, __Key__, __Value__> AppNamespace<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__AppNamespaceComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
fn main() {}
