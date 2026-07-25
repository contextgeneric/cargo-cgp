#![feature(prelude_import)]
//! A use-site `E0277` on a `Code`-dispatched handler pipeline whose root cause is a missing
//! **input-dispatch entry**: the third stage dispatches its provider on the *input type* through
//! [`UseInputDelegate`](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/reference/providers/use_delegate.md),
//! and no entry matches the type the second stage produces. This is the counterpart of the
//! missing-wiring leaf for the input axis: a delegation table (`SinkHandlers`) that is missing one
//! key, rather than a context that does not wire a component.
//!
//! Distilled from the `hypershell` `http_checksum_native` example: a `StreamingHttpRequest`
//! (fixed output stream) piped to `Checksum<Sha256>` (producing the raw digest
//! `GenericArray<u8, Sha256::OutputSize>`) piped to `StreamToStdout`, whose `HandleToTokioAsyncRead`
//! adapter is an `UseInputDelegate` that dispatches the incoming value on its type into a writable
//! form — with entries for byte-like inputs (`Vec<u8>`, `String`, stream wrappers) but **none for a
//! raw `GenericArray`**. The intervening `BytesToHex` stage (which would have turned the digest into
//! a `String`) is removed, so the raw digest reaches the sink and matches no input-dispatch entry.
//! The self-contained analogue here is `SinkHandlers` having no entry for `Tagged<Bytes>`.
//!
//! The pipeline steps are `Code` tags wrapped in a `Call<Code>` forwarding handler, so a stage's
//! `::Output` (the next stage's input) resolves *through* `CanHandle` rather than off a provider
//! impl directly. `App` runs the fixed `Prog` program through the async, `Code`-dispatched `Handler`
//! component, dispatched to a `PipeHandlers` composition that matches the code unconditionally, so
//! the failure is an `E0277` on the call with no span on `App`'s definition — only the call-site
//! anchor can recover it. The call's input (`Vec::new()`) is not written syntactically, so the walk
//! seeds it as an unknown placeholder.
//!
//! Stage 1, `Call<Fetch>`, forwards to `HandleFetch`, whose fixed `ByteStream` output cannot be
//! shown under the placeholder input (its `where Input: Send` is rejected against a rigid
//! placeholder), so `<App as CanHandle<Fetch, _>>::Output` stalls and must be recovered as a fixed
//! projection. Stage 2, `Call<Digest<Sha>>`, forwards to a nested
//! `PipeHandlers<Product![HandleAdapt, HandleHash]>`; `HandleHash` produces the fixed projection
//! output `Tagged<<Sha as HasOutSize>::OutSize>` = `Tagged<Bytes>`, independent of its input. Stage 3,
//! `Call<Sink>`, forwards to the input-dispatcher `UseInputDelegate<SinkHandlers>`, whose table has
//! no entry for `Tagged<Bytes>` — so `SinkHandlers: DelegateComponent<Tagged<Bytes>>` is the real
//! root cause.
//!
//! The missing entry surfaces as an unmet `DelegateComponent` on the *dispatch table* `SinkHandlers`
//! rather than on the context. The resolver reports it as a [`CGP-E110`] `MissingDispatchEntry` leaf
//! — a non-context delegation table (an aggregate provider or a `UseDelegate`/`UseInputDelegate`
//! table) missing a key — so the block leads with `provider \`SinkHandlers\` does not contain any
//! delegate entry for \`Tagged<Bytes>\`` over the dependency chain, rather than declining to
//! `PipeHandlers`/`ComposeHandlers` plumbing with the cause nowhere. The tree also shows
//! `Tagged<Bytes>` flowing into the sink stage, so a reader sees exactly which type reached a stage
//! that cannot handle it.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{
    CanHandle, Handler, HandlerComponent, PipeHandlers, UseInputDelegate,
};
use cgp::prelude::*;
/// The whole program, a single `Code` tag wired to the pipeline below.
pub struct Prog;
/// The pipeline steps, as `Code` tags dispatched per code.
pub struct Fetch;
pub struct Digest<H>(pub PhantomData<H>);
pub struct Sink;
/// The inner stage of `Digest`'s nested pipeline — an adapter that passes the stream on.
pub struct Adapt;
/// A hasher marker carrying its output size as an associated type, so `Digest`'s output is a
/// projection (`Tagged<Sha::OutSize>`), the self-contained analogue of `GenericArray<u8, Sha256::OutputSize>`.
pub struct Sha;
pub trait HasOutSize {
    type OutSize;
}
impl HasOutSize for Sha {
    type OutSize = Bytes;
}
/// The digest output payload — deliberately *not* `AsRef<[u8]>`, standing in for the raw
/// `GenericArray` bytes that `StreamToStdout` (an `AsyncRead` sink) cannot accept.
pub struct Bytes;
/// A wrapper around the hasher's output-size type; the fixed output of the digest stage.
pub struct Tagged<T>(pub PhantomData<T>);
/// The stream-like output of stage 1 and the input the digest stage consumes.
pub struct ByteStream;
/// A forwarding handler: interprets *any* outer code by running the context's handler for the inner
/// `InCode`, forwarding its `Output`. Makes a stage's `::Output` resolve through `CanHandle`.
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
/// Stage 1: produces a *fixed* `ByteStream`, but its `Input: Send` dependency cannot be shown under
/// the call's unknown input, so `<App as CanHandle<Fetch, _>>::Output` stalls — even though its
/// output does not depend on the input.
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
/// The inner adapter of `Digest`'s nested pipeline: adapts whatever stream comes in, forwarding a
/// `ByteStream`. Generic over the code, like a stream-conversion step.
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for HandleAdapt
where
    __Context__: HasErrorType,
{
    type Output = ByteStream;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Code>,
        _input: Input,
    ) -> Result<ByteStream, <__Context__ as HasErrorType>::Error> {
        Ok(ByteStream)
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for HandleAdapt
where
    __Context__: HasErrorType,
{}
pub struct HandleAdapt;
/// Stage 2's real work: consumes the adapted stream and produces the *fixed* projection output
/// `Tagged<H::OutSize>`, independent of its input.
impl<__Context__, Input, H> Handler<__Context__, Digest<H>, Input> for HandleHash
where
    H: HasOutSize,
    __Context__: HasErrorType,
{
    type Output = Tagged<H::OutSize>;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Digest<H>>,
        _input: Input,
    ) -> Result<Tagged<H::OutSize>, <__Context__ as HasErrorType>::Error> {
        Ok(Tagged(PhantomData))
    }
}
impl<
    __Context__,
    Input,
    H,
> IsProviderFor<HandlerComponent, __Context__, (Digest<H>, Input)> for HandleHash
where
    H: HasOutSize,
    __Context__: HasErrorType,
{}
pub struct HandleHash;
/// Stage 3's actual writer: accepts a byte-like input and consumes it. Reached only once the
/// input-dispatcher below has picked it for a known input type.
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for HandleWriteBytes
where
    Input: AsRef<[u8]>,
    __Context__: HasErrorType,
{
    type Output = ();
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Code>,
        _input: Input,
    ) -> Result<(), <__Context__ as HasErrorType>::Error> {
        Ok(())
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for HandleWriteBytes
where
    Input: AsRef<[u8]>,
    __Context__: HasErrorType,
{}
pub struct HandleWriteBytes;
pub struct HandleSink;
impl DelegateComponent<HandlerComponent> for HandleSink {
    type Delegate = UseInputDelegate<SinkHandlers>;
}
impl<__Context__, __Params__> IsProviderFor<HandlerComponent, __Context__, __Params__>
for HandleSink
where
    UseInputDelegate<
        SinkHandlers,
    >: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
pub struct SinkHandlers;
impl DelegateComponent<Vec<u8>> for SinkHandlers {
    type Delegate = HandleWriteBytes;
}
impl<__Context__, __Params__> IsProviderFor<Vec<u8>, __Context__, __Params__>
for SinkHandlers
where
    HandleWriteBytes: IsProviderFor<Vec<u8>, __Context__, __Params__>,
{}
impl DelegateComponent<String> for SinkHandlers {
    type Delegate = HandleWriteBytes;
}
impl<__Context__, __Params__> IsProviderFor<String, __Context__, __Params__>
for SinkHandlers
where
    HandleWriteBytes: IsProviderFor<String, __Context__, __Params__>,
{}
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
impl<__Table__, H, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Digest<H>, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = PipeHandlers<Product![HandleAdapt, HandleHash]>;
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
    type Delegate = PipeHandlers<
        Product![Call<Fetch>, Call<Digest<Sha>>, Call<Sink>],
    >;
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
