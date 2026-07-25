#![feature(prelude_import)]
//! Acceptable failure: a `for` loop that wires a **bare key** (`Key: Value`)
//! instead of embedding it in a path, in a context that also joins a namespace —
//! the two blanket impls overlap and the compiler rejects them with `E0119`.
//!
//! `namespace DefaultNamespace;` emits a blanket `impl<Key, Value>
//! DelegateComponent<Key> for App where Key: DefaultNamespace<App, ..>` (plus the
//! matching `IsProviderFor` forwarding) that covers *every* key. A `for <Key, Value>
//! in GreeterTable { Key: Value }` loop emits a second blanket `impl<Key, Value>
//! DelegateComponent<Key> for App where Key: GreeterTable<App, ..>` — also over every
//! key — and the two overlap because a key could satisfy both `where` clauses, so
//! coherence rejects the pair (`E0119`, fully generic `DelegateComponent<_>` /
//! `IsProviderFor<_, _, _>`). This is why a loop key must sit inside a path
//! (`@app.SomeComponent.Key: Value`), which keys the impl on a concrete path rather
//! than on every key. CGP lowers both blanket impls faithfully; only the whole
//! program reveals the overlap, so it defers to the compiler.
//!
//! This is the blanket-vs-blanket shape of the overlapping-forwarding class,
//! alongside two_namespaces_joined.rs (two `namespace` joins on one context);
//! contrast the specific-vs-blanket override in override_registered_path.rs.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-forwarding-conflict.md.
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
pub struct __GreeterTableComponents;
pub trait GreeterTable<__Table__> {
    type Delegate;
}
impl<__Table__> GreeterTable<__Table__> for GreeterComponent {
    type Delegate = GreetHello;
}
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<Key, Value> DelegateComponent<Key> for App
where
    Key: GreeterTable<App, Delegate = Value>,
{
    type Delegate = Value;
}
impl<Key, Value, __Context__, __Params__> IsProviderFor<Key, __Context__, __Params__>
for App
where
    Key: GreeterTable<App, Delegate = Value>,
    Value: IsProviderFor<Key, __Context__, __Params__>,
{}
fn main() {}
