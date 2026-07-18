//! Usability: a resolved dispatch chain repeats the full program type at every node.
//!
//! `App` runs a four-stage pipeline program through the `Code`-dispatched `Handler`
//! component, inherited through two namespace levels. The first stage reads the
//! context's `name` field, which `App` lacks, and the call-site anchor resolves the
//! failure cleanly — root cause first, one block. What this fixture pins is the
//! *presentation* of that success on a DSL-sized program: every `Handler` node in
//! the dependency chain restates the entire `Prog<Product![…]>` code type, and each
//! namespace level adds a redirect hop, so the tree grows both long and wide even
//! though most nodes differ only in the provider column. A realistic DSL context
//! produces dozens of such nodes with a far larger program type. Eliding the
//! unchanged code parameter after its first appearance — or folding uninformative
//! redirect hops — is the open work. See docs/issues/usability.md.

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
