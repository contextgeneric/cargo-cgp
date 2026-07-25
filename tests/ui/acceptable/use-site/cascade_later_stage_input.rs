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
//! `cgp-knowledge-base/cargo-cgp/implementation/typed-root-cause-resolution.md`.

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
#[async_trait]
#[cgp_impl(new Call<InCode>)]
impl<OutCode, InCode, Input, Output> Handler<OutCode, Input>
where
    Self: CanHandle<InCode, Input, Output = Output>,
{
    type Output = Output;

    async fn handle(&self, _tag: PhantomData<OutCode>, input: Input) -> Result<Output, Self::Error> {
        self.handle(PhantomData::<InCode>, input).await
    }
}

/// First pipe stage's handler: produces a *fixed* `ByteStream`, but its `Input: Send` dependency
/// cannot be shown to hold under the call's unknown input, so `<App as CanHandle<Fetch, _>>::Output`
/// never normalizes — even though that output type does not depend on the input at all.
#[async_trait]
#[cgp_impl(new HandleFetch)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<Fetch, Input>
where
    Input: Send,
{
    type Output = ByteStream;

    async fn handle(&self, _tag: PhantomData<Fetch>, _input: Input) -> Result<ByteStream, Error> {
        Ok(ByteStream)
    }
}

/// Second pipe stage's handler: requires its input to be byte-like (`Input: AsRef<[u8]>`). Its
/// input is the first stage's fixed `ByteStream` output, which is not `AsRef<[u8]>` — the real
/// root cause.
#[async_trait]
#[cgp_impl(new HandleHex)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<Hex, Input>
where
    Input: AsRef<[u8]>,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Hex>, input: Input) -> Result<String, Error> {
        Ok(hex_of(input.as_ref()))
    }
}

fn hex_of(_bytes: &[u8]) -> String {
    String::new()
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        @cgp.extra.handler.HandlerComponent.Fetch:
            HandleFetch,
        @cgp.extra.handler.HandlerComponent.Hex:
            HandleHex,
        @cgp.extra.handler.HandlerComponent.Prog:
            PipeHandlers<Product![Call<Fetch>, Call<Hex>]>,
    }
}

#[derive(HasField)]
pub struct App {}

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Prog>, Vec::new()).await?;
    Ok(())
}

fn main() {
    let _ = run_app;
}
