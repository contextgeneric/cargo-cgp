#![feature(prelude_import)]
//! A use-site `E0277` on a handler pipeline whose root cause is an *empty* input-dispatch table: the
//! second stage dispatches its provider on the input type through `UseInputDelegate<EmptySink>`, and
//! that table wires no entries at all. This pins the **structural** half of the missing-dispatch-entry
//! recognition, the case the owner-property check (`is_delegation_table`, "does the table wire *some*
//! key") cannot see — an empty table has no `DelegateComponent` impl to find.
//!
//! The resolver instead keys on *where* the unmet `DelegateComponent` arises: it is a `where`-clause
//! of `UseInputDelegate`'s own provider impl, a lookup into a *separate* table (`EmptySink` is a
//! parameter of `UseInputDelegate<EmptySink>`), so it is unambiguously a missing dispatch entry —
//! whether or not that table happens to wire any other key. The block therefore leads with
//! `[CGP-E110] provider \`EmptySink\` does not contain any delegate entry for \`ByteStream\`` rather
//! than declining to raw `PipeHandlers`/`ComposeHandlers` plumbing.
//!
//! Stage 1, `Call<Fetch>`, produces a fixed `ByteStream` (its `where Input: Send` stalls under the
//! call's unknown input, so the output is recovered as a fixed projection). Stage 2, `Call<Sink>`,
//! forwards to `UseInputDelegate<EmptySink>`; its input is stage 1's `ByteStream`, for which the
//! empty table has no entry — `EmptySink: DelegateComponent<ByteStream>` is the root cause.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{
    CanHandle, Handler, HandlerComponent, PipeHandlers, UseInputDelegate,
};
use cgp::prelude::*;
pub struct Prog;
pub struct Fetch;
pub struct Sink;
/// The stream-like output of stage 1 and the input the sink dispatches on.
pub struct ByteStream;
/// A forwarding handler: interprets any outer code by running the context's handler for the inner
/// `InCode`, so a stage's `::Output` resolves through `CanHandle`.
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
/// Stage 1: produces a fixed `ByteStream`, gated on `Input: Send` so it stalls under the call's
/// unknown input and its output must be recovered as a fixed projection.
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
pub struct HandleSink;
impl DelegateComponent<HandlerComponent> for HandleSink {
    type Delegate = UseInputDelegate<EmptySink>;
}
impl<__Context__, __Params__> IsProviderFor<HandlerComponent, __Context__, __Params__>
for HandleSink
where
    UseInputDelegate<
        EmptySink,
    >: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
pub struct EmptySink;
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
            PathCons<HandlerComponent, PathCons<Sink, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = HandleSink;
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
    type Delegate = PipeHandlers<Product![Call<Fetch>, Call<Sink>]>;
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
