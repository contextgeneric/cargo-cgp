//! Usability: a use-site wiring failure whose downstream `?`-operator cascade
//! pollutes the log with unreadable projected types.
//!
//! `App` runs a `MyProgram` program through the async, `Code`-dispatched `Handler`
//! component, dispatched to a `PipeHandlers` composition (the same combinator
//! plumbing a real DSL context produces). The first stage, `HandleName`, reads the
//! context's `name` field, but `App` has none, so awaiting `app.handle(..)` at a
//! use site fails. Because the composition matches the `Code` unconditionally the
//! method is *found* (its trait bound is only conditionally satisfied), so the
//! failure is an `E0277`, not an `E0599`, and its span never lands on `App`'s type
//! definition — so the typed resolver cannot anchor the context and declines to the
//! text rewrite, which exposes the internal `PipeHandlers`/`ComposeHandlers`
//! plumbing.
//!
//! The pollution this fixture pins is the *cascade*: once `handle`'s return type is
//! unresolvable, the trailing `?` produces `the ? operator ...` / `Try` /
//! `FromResidual` errors that dump the giant `<App as CanHandle<..>>::Output`
//! projection and add nothing over the wiring failure already reported at the same
//! span. See docs/issues/usability.md.

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;

/// A program wrapper generic over its pipeline steps — the analogue of the DSL's
/// `Pipe<Steps>`, wired to a single provider that matches *every* `Prog<Steps>`.
pub struct Prog<Steps>(pub PhantomData<Steps>);

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// First pipe stage: produce the context's name. Depends on `Self: HasName`, which
/// `App` cannot meet — the buried root cause.
#[async_trait]
#[cgp_impl(new HandleName)]
#[use_type(HasErrorType.Error)]
impl<Code, Input> Handler<Code, Input>
where
    Self: HasName,
    Input: Send,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Code>, _input: Input) -> Result<String, Error> {
        Ok(self.name().to_owned())
    }
}

/// Second pipe stage: uppercase the name. Takes the first stage's `String` output.
#[async_trait]
#[cgp_impl(new HandleShout)]
#[use_type(HasErrorType.Error)]
impl<Code> Handler<Code, String>
where
    Self: Sync,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Code>, input: String) -> Result<String, Error> {
        Ok(input.to_uppercase())
    }
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        @cgp.extra.handler.HandlerComponent.<Steps> Prog<Steps>:
            PipeHandlers<Steps>,
    }
}

#[derive(HasField)]
pub struct App {
    // No `name` field — `HandleName`'s `Self: HasName` dependency cannot be met.
}

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Prog<Product![HandleName, HandleShout]>>, Vec::new())
        .await?;
    Ok(())
}

fn main() {
    let _ = run_app;
}
