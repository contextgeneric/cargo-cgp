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
//! unresolved `::Output`. A walk that dropped a `ComposeHandlers` stage keyed on a projection
//! carrying an inference variable would descend only the first stage (a mere `_: Send` it cannot
//! report) and decline. The walk instead folds such a stray variable into a rigid placeholder, so
//! it descends into `HandleName` and reaches the missing `name` field, while the first stage's
//! `_: Send`, which genuinely depends on the input, is filtered out as a placeholder leaf.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E103 CGP-E104 CGP-E110

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;

/// A program wrapper generic over its pipeline steps, wired to a single provider that matches
/// *every* `Prog<Steps>`.
pub struct Prog<Steps>(pub PhantomData<Steps>);

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// First pipe stage: depends only on `Input: Send`. With the call's unknown input this cannot be
/// shown to hold, so its `::Output` (used as the next stage's input) never normalizes — the shape
/// that used to hide a cause living past the first stage.
#[async_trait]
#[cgp_impl(new HandleConnect)]
#[use_type(HasErrorType.Error)]
impl<Code, Input> Handler<Code, Input>
where
    Input: Send,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Code>, _input: Input) -> Result<String, Error> {
        Ok(String::new())
    }
}

/// Second pipe stage: reads the context's `name` field — the buried root cause. Its input is the
/// first stage's unresolved `::Output`; because it accepts *any* input (like a real DSL's later
/// handlers), the walk can descend it once that input is folded to a placeholder, and reach the
/// context-side `Self: HasName` dependency past the `Input: Send` one it cannot report.
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
    app.handle(PhantomData::<Prog<Product![HandleConnect, HandleName]>>, Vec::new())
        .await?;
    Ok(())
}

fn main() {
    let _ = run_app;
}
