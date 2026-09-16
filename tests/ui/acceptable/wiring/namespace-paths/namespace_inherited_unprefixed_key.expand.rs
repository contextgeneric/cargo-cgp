#![feature(prelude_import)]
//! Acceptable failure: a namespace inherits a parent and then binds a bare, unprefixed
//! component key that the parent already *redirects* — the namespace-level counterpart
//! of namespace_unprefixed_key.rs, where a *context* makes the same mistake.
//!
//! `#[prefix(@app in DefaultNamespace)]` registers `GreeterComponent` in
//! `DefaultNamespace` as a `RedirectLookup` down `@app.GreeterComponent`, and
//! `new AppNamespace: DefaultNamespace` emits the inheritance blanket impl
//! `impl<Table, Key, Value> AppNamespace<Table> for Key where Key: DefaultNamespace<…>`,
//! which forwards every key the parent answers — `GreeterComponent` among them.
//! `#[default_impl(GreeterComponent in AppNamespace)]` then emits a second impl
//! `impl<Table> AppNamespace<Table> for GreeterComponent`, and the two overlap, so
//! coherence rejects the pair (`E0119`, a *single* conflict on
//! `AppNamespace<_> for GreeterComponent`, since a namespace emits only its own
//! lookup-trait impl, not the context-side `DelegateComponent`/`IsProviderFor` pair).
//!
//! The key is the mistake: a prefixed component is addressed by its *path*, so the
//! registration must read `@app.GreeterComponent`. The tool recovers that target by
//! normalizing `<GreeterComponent as DefaultNamespace<AppNamespace's table>>::Delegate`
//! through the trait solver, exactly as it does for the context-level shape, so this
//! reads as a *redirect collision* (`[CGP-E007]`) naming the path to write instead.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E007).
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
impl<__Components__> DefaultNamespace<__Components__> for GreeterComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@app.GreeterComponent)>;
}
pub struct __AppNamespaceComponents;
pub trait AppNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> AppNamespace<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__AppNamespaceComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        "Hello".to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
impl<__Components__> AppNamespace<__Components__> for GreeterComponent {
    type Delegate = GreetHello;
}
fn main() {}
