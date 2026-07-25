#![feature(prelude_import)]
//! A use-site `E0277` on a `Code`-dispatched handler pipeline whose root cause lives in a
//! *later* stage — the shape [`cascade_after_use_site`](cascade_after_use_site.rs) does not cover.
//!
//! As there, `App` runs a program through the async, `Code`-dispatched `Handler` component,
//! dispatched to a `PipeHandlers` composition that matches the `Code` unconditionally, so the
//! failure is an `E0277` on the call with no span landing on `App`'s definition — only the
//! call-site anchor can recover it.
//!
//! The difference is *where* the cause sits. Here the **first** stage, `HandleConnect`, depends
//! only on `Input: Send`; with the call's unknown (placeholder) input that bound cannot be shown
//! to hold, so `HandleConnect`'s `::Output` never normalizes. The **second** stage, `HandleName`,
//! reads the context's `name` field — the real buried cause — and its input is that first stage's
//! unresolved `::Output`. Because a `ComposeHandlers` stage keyed on an earlier stage's `::Output`
//! is dropped when that projection carries an inference var, the walk used to descend only the
//! first stage (a mere `_: Send` it cannot report) and decline, falling back to three
//! `PipeHandlers`/`ComposeHandlers`-plumbing `[CGP-E002]` blocks with the cause nowhere. The walk
//! now folds such a stray var into a rigid placeholder rather than dropping the stage, so it
//! descends into `HandleName` and reaches the missing `name` field, while the first stage's
//! `_: Send` — genuinely input-dependent — is filtered out as a placeholder leaf.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;
/// A program wrapper generic over its pipeline steps, wired to a single provider that matches
/// *every* `Prog<Steps>`.
pub struct Prog<Steps>(pub PhantomData<Steps>);
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
/// First pipe stage: depends only on `Input: Send`. With the call's unknown input this cannot be
/// shown to hold, so its `::Output` (used as the next stage's input) never normalizes — the shape
/// that used to hide a cause living past the first stage.
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for HandleConnect
where
    Input: Send,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Code>,
        _input: Input,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(String::new())
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for HandleConnect
where
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandleConnect;
/// Second pipe stage: reads the context's `name` field — the buried root cause. Its input is the
/// first stage's unresolved `::Output`; because it accepts *any* input (like a real DSL's later
/// handlers), the walk can descend it once that input is folded to a placeholder, and reach the
/// context-side `Self: HasName` dependency past the `Input: Send` one it cannot report.
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for HandleName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Code>,
        _input: Input,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(__context__.name().to_owned())
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for HandleName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandleName;
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
impl<__Table__, Steps, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Prog<Steps>, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = PipeHandlers<Steps>;
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
    app.handle(PhantomData::<Prog<Product![HandleConnect, HandleName]>>, Vec::new())
        .await?;
    Ok(())
}
fn main() {
    let _ = run_app;
}
