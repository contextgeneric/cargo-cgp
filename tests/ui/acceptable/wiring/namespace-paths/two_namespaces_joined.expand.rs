#![feature(prelude_import)]
//! Acceptable failure: a context that joins **two** namespaces at once —
//! `namespace NamespaceA; namespace NamespaceB;` — cannot compile, because each
//! join emits a *blanket* forwarding impl over every key and the two overlap.
//!
//! Each `namespace N;` header emits `impl<Key, Value> DelegateComponent<Key> for
//! App where Key: N<App, ..>` (plus the matching `IsProviderFor` forwarding), a
//! blanket impl that covers *every* key. Joining two namespaces emits two such
//! blanket impls — one keyed through `NamespaceA`, one through `NamespaceB` — and
//! because a key could satisfy both `where` clauses, coherence cannot prove they
//! never overlap and rejects the pair (`E0119`, fully generic `DelegateComponent<_>`
//! / `IsProviderFor<_, _, _>`, carets on the two `namespace` lines, no downstream
//! note). A context therefore forwards through at most one namespace; layer several
//! by having that one namespace *inherit* the others (`new Combined: A { .. }`
//! inheriting further), not by joining several on the context. CGP lowers both
//! blanket impls faithfully; only the whole program reveals the overlap, so it
//! defers to the compiler.
//!
//! This is the blanket-vs-blanket shape of the overlapping-forwarding class,
//! alongside for_loop_bare_key.rs (a namespace join plus a bare-key `for` loop);
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
pub struct __NamespaceAComponents;
pub trait NamespaceA<__Table__> {
    type Delegate;
}
impl<__Table__> NamespaceA<__Table__> for GreeterComponent {
    type Delegate = GreetHello;
}
pub struct __NamespaceBComponents;
pub trait NamespaceB<__Table__> {
    type Delegate;
}
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: NamespaceA<App, Delegate = __Value__>,
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
    __Key__: NamespaceA<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: NamespaceB<App, Delegate = __Value__>,
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
    __Key__: NamespaceB<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
fn main() {}
