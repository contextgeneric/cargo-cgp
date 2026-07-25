#![feature(prelude_import)]
//! A pipeline stage that routes back to the context for a `Code` nothing is wired for.
//!
//! This is the shape a type-level DSL produces when a program names a syntax the language
//! has no interpreter for — most simply, when a step is deleted and something else is left
//! in its place. Each stage of the pipeline is a `Call<Code>` provider whose whole job is to
//! route back through the context's *own* handler for its `Code`, so a stage naming an
//! unwired `Code` fails not on a dependency of its own but on a missing dispatch entry one
//! hop away. Here `Missing` has no entry in the namespace, so `Call<Missing>` cannot resolve.
//!
//! The resolver used to decline it, leaving three `[CGP-E002]` blocks — one per combinator
//! layer rustc happened to report — each headed by the `PipeHandlers` or `ComposeHandlers`
//! plumbing the programmer never wrote, restating the whole program type, and none of them
//! naming `Missing` at all. What defeated the walk was the namespace join: it gives the context
//! a blanket `DelegateComponent<__Key__>` forwarding that unifies with *every* key, so the
//! missing entry looked like a node to descend, and the descent reached the namespace's own
//! lookup machinery rather than a cause.
//!
//! An unmet delegation on the context is now terminal whatever nominally matches it — had the
//! context wired the key, the obligation would hold and be pruned before becoming a node at all
//! — so this resolves to one `[CGP-E001]` block over a `[CGP-E107]` root cause naming the
//! absent `@…HandlerComponent.Missing` entry, with the whole combinator chain beneath it.
//!
//! It also pins the trailing-segment trim. A `RedirectLookup` keys on the path *plus* the
//! component's parameters, and this failure is recovered from a call whose input is inferred, so
//! the raw key ends in a placeholder segment; the leaf names `@….Missing` — the entry the
//! programmer can actually write — rather than an unwritable key that would be dropped as
//! unknowable.
//!
//! Distilled from a real DSL pipeline whose first stage was removed; the counterpart where the
//! stages are providers wired directly rather than routed back through the context is
//! `acceptable/use-site/cascade_after_use_site.rs`.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;
/// A program: a pipeline of syntax fragments, each interpreted by the context.
pub struct Prog;
/// A syntax fragment the wiring *does* interpret.
pub struct Present;
/// A syntax fragment the wiring does **not** interpret — the root cause.
pub struct Missing;
/// The DSL's indirection: a stage that interprets its `OutCode` by routing back through the
/// context's own handler for `InCode`. A stage naming an unwired fragment fails here.
impl<__Context__, OutCode, InCode, Input, Output> Handler<__Context__, OutCode, Input>
for Call<InCode>
where
    __Context__: CanHandle<InCode, Input, Output = Output>,
    __Context__: HasErrorType,
{
    type Output = Output;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<OutCode>,
        input: Input,
    ) -> Result<Output, <__Context__ as HasErrorType>::Error> {
        __context__.handle(PhantomData::<InCode>, input).await
    }
}
impl<
    __Context__,
    OutCode,
    InCode,
    Input,
    Output,
> IsProviderFor<HandlerComponent, __Context__, (OutCode, Input)> for Call<InCode>
where
    __Context__: CanHandle<InCode, Input, Output = Output>,
    __Context__: HasErrorType,
{}
pub struct Call<InCode>(pub ::core::marker::PhantomData<InCode>);
/// The interpreter for `Present`.
impl<__Context__, Input> Handler<__Context__, Present, Input> for HandlePresent
where
    Input: Send,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Present>,
        _input: Input,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok("present".to_owned())
    }
}
impl<__Context__, Input> IsProviderFor<HandlerComponent, __Context__, (Present, Input)>
for HandlePresent
where
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandlePresent;
pub struct __MyNamespaceComponents;
pub trait MyNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> MyNamespace<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__MyNamespaceComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("core"),
        PathCons<Symbol!("error"), PathCons<ErrorTypeProviderComponent, __Wildcard__>>,
    >,
> {
    type Delegate = UseType<String>;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Prog, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = PipeHandlers<Product![Call<Missing>, Call<Present>]>;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Present, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = HandlePresent;
}
pub struct App {}
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: MyNamespace<App, Delegate = __Value__>,
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
    __Key__: MyNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Prog>, Vec::new()).await?;
    Ok(())
}
fn main() {
    let _ = run_app;
}
