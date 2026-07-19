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

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers, UseInputDelegate};
use cgp::prelude::*;

pub struct Prog;

pub struct Fetch;
pub struct Sink;

/// The stream-like output of stage 1 and the input the sink dispatches on.
pub struct ByteStream;

/// A forwarding handler: interprets any outer code by running the context's handler for the inner
/// `InCode`, so a stage's `::Output` resolves through `CanHandle`.
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

/// Stage 1: produces a fixed `ByteStream`, gated on `Input: Send` so it stalls under the call's
/// unknown input and its output must be recovered as a fixed projection.
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

// Stage 2, `Sink`, dispatches on its input type through an empty table — no entry for any input.
delegate_components! {
    new HandleSink {
        HandlerComponent:
            UseInputDelegate<EmptySink>,
    }
}

delegate_components! {
    new EmptySink {}
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        @cgp.extra.handler.HandlerComponent.Fetch:
            HandleFetch,
        @cgp.extra.handler.HandlerComponent.Sink:
            HandleSink,
        @cgp.extra.handler.HandlerComponent.Prog:
            PipeHandlers<Product![Call<Fetch>, Call<Sink>]>,
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
