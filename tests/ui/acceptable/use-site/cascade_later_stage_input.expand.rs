#![feature(prelude_import)]
//! A use-site `E0277` on a `Code`-dispatched handler pipeline whose root cause is a *later*
//! stage's requirement on its **input type** — where that input is an earlier stage's fixed
//! output, threaded through a forwarding `Call` handler. This is the shape
//! [`cascade_later_stage`](cascade_later_stage.rs) does not cover: there the later stage's reported
//! cause is *context-side* (a missing field, independent of the input), so folding the unknown
//! pipeline input to a placeholder and descending still reaches it. Here the only cause *is* the
//! input.
//!
//! The pipeline steps are `Code` tags (`Fetch`, `Hex`) wrapped in a `Call<Code>` forwarding
//! handler — the shape a handler DSL uses so a pipeline of code tags becomes a pipeline of
//! providers. `Call<InCode>` handles *any* outer code by dispatching to the context's own
//! `CanHandle<InCode, Input>`, forwarding that inner handler's `Output` as its own. So a stage's
//! `::Output` (which the next stage takes as its input) is resolved *through* `CanHandle`, not read
//! off a provider impl directly.
//!
//! `App` runs the fixed `Prog` program through the async, `Code`-dispatched `Handler` component,
//! dispatched to a `PipeHandlers` composition that matches the code unconditionally, so the failure
//! is an `E0277` on the call with no span landing on `App`'s definition — only the call-site anchor
//! can recover it. The call's input (`Vec::new()`) is not written syntactically, so the walk seeds
//! it as an unknown placeholder.
//!
//! The first stage, `Call<Fetch>`, forwards to `HandleFetch`, which has a **fixed** output type
//! (`ByteStream`) but a `where Input: Send` dependency that cannot be shown to hold under the
//! placeholder input — so against a rigid placeholder its impl is *rejected* and
//! `<App as CanHandle<Fetch, _>>::Output` never normalizes, even though that output type does not
//! depend on the input at all. The second stage, `Call<Hex>`, forwards to `HandleHex`, which
//! requires `Input: AsRef<[u8]>`; its input is the first stage's `::Output` — a fixed `ByteStream`,
//! which is *not* `AsRef<[u8]>`: the real root cause.
//!
//! The resolver recovers the fixed output rather than folding it to a placeholder, so the second
//! stage's input becomes the concrete `ByteStream` and its `ByteStream: AsRef<[u8]>` requirement is
//! reported as the root cause; before that recovery it declined to
//! `PipeHandlers`/`ComposeHandlers`-plumbing `[CGP-E002]` blocks with the cause nowhere in the
//! output. The mechanism (and why it generalizes to any stalled associated-type projection) is
//! documented under "Walking the dependency graph downward" in
//! `docs/implementation/typed-root-cause-resolution.md`.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;
/// The whole program, a single `Code` tag wired to the pipeline below.
pub struct Prog;
/// The pipeline steps, as `Code` tags dispatched per code.
pub struct Fetch;
pub struct Hex;
/// A stream-like output type that is deliberately *not* `AsRef<[u8]>` — it stands in for the
/// `Pin<Box<dyn AsyncRead>>` an HTTP response streams as.
pub struct ByteStream;
/// A forwarding handler: interprets *any* outer code by running the context's handler for the
/// inner `InCode`, forwarding its `Output`. This is the pipeline-step wrapper shape a handler DSL
/// uses, and it is what makes a stage's `::Output` resolve *through* `CanHandle`.
impl<__Context__, OutCode, InCode, Input, Output> Handler<__Context__, OutCode, Input>
for Call<InCode>
where
    __Context__: CanHandle<InCode, Input, Output = Output>,
{
    type Output = Output;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<OutCode>,
        input: Input,
    ) -> Result<Output, __Context__::Error> {
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
{}
pub struct Call<InCode>(pub ::core::marker::PhantomData<InCode>);
/// First pipe stage's handler: produces a *fixed* `ByteStream`, but its `Input: Send` dependency
/// cannot be shown to hold under the call's unknown input, so `<App as CanHandle<Fetch, _>>::Output`
/// never normalizes — even though that output type does not depend on the input at all.
impl<__Context__, Input> Handler<__Context__, Fetch, Input> for HandleFetch
where
    Input: Send,
    __Context__: HasErrorType,
{
    type Output = ByteStream;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Fetch>,
        _input: Input,
    ) -> Result<ByteStream, <__Context__ as HasErrorType>::Error> {
        Ok(ByteStream)
    }
}
impl<__Context__, Input> IsProviderFor<HandlerComponent, __Context__, (Fetch, Input)>
for HandleFetch
where
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandleFetch;
/// Second pipe stage's handler: requires its input to be byte-like (`Input: AsRef<[u8]>`). Its
/// input is the first stage's fixed `ByteStream` output, which is not `AsRef<[u8]>` — the real
/// root cause.
impl<__Context__, Input> Handler<__Context__, Hex, Input> for HandleHex
where
    Input: AsRef<[u8]>,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Hex>,
        input: Input,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(hex_of(input.as_ref()))
    }
}
impl<__Context__, Input> IsProviderFor<HandlerComponent, __Context__, (Hex, Input)>
for HandleHex
where
    Input: AsRef<[u8]>,
    __Context__: HasErrorType,
{}
pub struct HandleHex;
fn hex_of(_bytes: &[u8]) -> String {
    String::new()
}
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
            PathCons<HandlerComponent, PathCons<Fetch, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = HandleFetch;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Hex, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = HandleHex;
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
    type Delegate = PipeHandlers<Product![Call<Fetch>, Call<Hex>]>;
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
