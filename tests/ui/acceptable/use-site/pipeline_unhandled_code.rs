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
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md

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
#[async_trait]
#[cgp_impl(new Call<InCode>)]
#[use_type(HasErrorType.Error)]
impl<OutCode, InCode, Input, Output> Handler<OutCode, Input>
where
    Self: CanHandle<InCode, Input, Output = Output>,
{
    type Output = Output;

    async fn handle(&self, _tag: PhantomData<OutCode>, input: Input) -> Result<Output, Error> {
        self.handle(PhantomData::<InCode>, input).await
    }
}

/// The interpreter for `Present`.
#[async_trait]
#[cgp_impl(new HandlePresent)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<Present, Input>
where
    Input: Send,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Present>, _input: Input) -> Result<String, Error> {
        Ok("present".to_owned())
    }
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        // The program runs as a pipeline of `Call` stages, one per fragment.
        @cgp.extra.handler.HandlerComponent.Prog:
            PipeHandlers<Product![Call<Missing>, Call<Present>]>,

        // `Present` is interpreted; `Missing` is not wired at all.
        @cgp.extra.handler.HandlerComponent.Present:
            HandlePresent,
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
