//! A resolved dispatch chain over a DSL-sized program, its repeated code type elided.
//!
//! `App` runs a four-stage pipeline program through the `Code`-dispatched `Handler`
//! component, inherited through two namespace levels. The first stage reads the
//! context's `name` field, which `App` lacks, and the call-site anchor resolves the
//! failure cleanly — root cause first, one block. What this fixture pins is the
//! *presentation* of that success on a program-sized `Code` type: the full
//! `Prog<Product![…]>` type appears once, on the first `Handler` node (and the
//! header), and every subsequent hop that restates the same trait and parameters is
//! elided to `Handler<…>`, so the chain reads as its meaningful steps — the pipeline
//! unfolding stage by stage down to the missing field — rather than as dozens of
//! near-identical lines.
//!
//! CGP error class:
//! ../../../../../cgp/docs/errors/checks/check-trait-failure.md (use-site face).

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;

/// A program wrapper generic over its pipeline steps — the analogue of a DSL's
/// `Pipe<Steps>`, wired to a single provider that matches *every* `Prog<Steps>`.
pub struct Prog<Steps>(pub PhantomData<Steps>);

/// Style markers, so the pass-through stages carry DSL-sized names into every label.
pub struct Politely;
pub struct Shout;
pub struct Repeat;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// First pipe stage: produce the context's name. Depends on `Self: HasName`, which
/// `App` cannot meet — the root cause.
#[async_trait]
#[cgp_impl(new HandleReadName)]
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

/// A pass-through stage, generic over a style marker, so the pipeline has depth and
/// each step a distinct, DSL-sized name.
#[async_trait]
#[cgp_impl(new HandlePass<Style>)]
#[use_type(HasErrorType.Error)]
impl<Code, Style> Handler<Code, String>
where
    Self: Sync,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<Code>, input: String) -> Result<String, Error> {
        Ok(input)
    }
}

cgp_namespace! {
    new BaseNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        @cgp.extra.handler.HandlerComponent.<Steps> Prog<Steps>:
            PipeHandlers<Steps>,
    }
}

cgp_namespace! {
    new ExtendedNamespace: BaseNamespace {
    }
}

#[derive(HasField)]
pub struct App {
    // No `name` field — `HandleReadName`'s `Self: HasName` dependency cannot be met.
}

delegate_components! {
    App {
        namespace ExtendedNamespace;
    }
}

pub type Program = Prog<
    Product![
        HandleReadName,
        HandlePass<Politely>,
        HandlePass<Shout>,
        HandlePass<Repeat>,
    ],
>;

async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Program>, Vec::new()).await?;
    Ok(())
}

fn main() {
    let _ = run_app;
}
