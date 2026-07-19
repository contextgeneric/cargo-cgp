//! A call-site `E0277` whose failing provider **destructures its input on a tuple shape**, with the
//! root cause reachable only once that shape is recovered — the case
//! [`cascade_later_stage`](cascade_later_stage.rs) and [`cascade_after_use_site`](cascade_after_use_site.rs)
//! do not cover.
//!
//! `Branch<Cond>` is a `Code`-dispatched handler whose interpreter `HandleBranch` takes a *tuple*
//! input `(CondInput, Rest)`, running the inner `Cond` on the first component. The program
//! `Branch<ReadName>` runs `ReadName`, which reads the context's `name` field — the buried cause,
//! since `App` has none. The call passes a tuple literal `(Vec::new(), Vec::new())` whose element
//! types the call does not write.
//!
//! The wiring matches the `Code` unconditionally, so the method is *found* and the failure is an
//! `E0277` on the call with no span on `App`'s definition — only the call-site anchor applies. That
//! anchor seeds the call's input from its written argument types; an all-unknown tuple used to
//! collapse to one flat placeholder, which cannot destructure into `HandleBranch`'s
//! `(CondInput, Rest)`, so its impl never matched and the resolver declined to three
//! `PipeHandlers`/`ComposeHandlers`-plumbing `[CGP-E002]` blocks. The anchor now recovers the tuple
//! *shape* (each unwritten element a placeholder), so `HandleBranch` matches, the walk runs the
//! inner `Cond`, and it reaches the missing `name` field.

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent};
use cgp::prelude::*;

/// A conditional program that runs its inner `Cond` on the first component of a tuple input.
pub struct Branch<Cond>(pub PhantomData<Cond>);

/// Inner program marker: read the context's `name`.
pub struct ReadName;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// Interpreter for `ReadName`: reads the context's `name` field — the buried root cause.
#[async_trait]
#[cgp_impl(new HandleReadName)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<ReadName, Input>
where
    Self: HasName,
    Input: Send,
{
    type Output = String;

    async fn handle(&self, _tag: PhantomData<ReadName>, _input: Input) -> Result<String, Error> {
        Ok(self.name().to_owned())
    }
}

/// Interpreter for `Branch<Cond>`: destructures its input as a 2-tuple and runs `Cond` on the first
/// component. Its input *shape* is a tuple, so a call that does not type that tuple must still have
/// its structure recovered for this impl to match.
#[async_trait]
#[cgp_impl(new HandleBranch)]
#[use_type(HasErrorType.Error)]
impl<Cond, CondInput, Rest> Handler<Branch<Cond>, (CondInput, Rest)>
where
    Self: CanHandle<Cond, CondInput, Output = String>,
    Rest: Send,
{
    type Output = String;

    async fn handle(
        &self,
        _tag: PhantomData<Branch<Cond>>,
        (cond_input, _rest): (CondInput, Rest),
    ) -> Result<String, Error> {
        self.handle(PhantomData::<Cond>, cond_input).await
    }
}

cgp_namespace! {
    new MyNamespace: DefaultNamespace {
        @cgp.core.error.ErrorTypeProviderComponent:
            UseType<String>,

        @cgp.extra.handler.HandlerComponent.<Cond> Branch<Cond>:
            HandleBranch,

        @cgp.extra.handler.HandlerComponent.ReadName:
            HandleReadName,
    }
}

#[derive(HasField)]
pub struct App {
    // No `name` field — `HandleReadName`'s `Self: HasName` dependency cannot be met.
}

delegate_components! {
    App {
        namespace MyNamespace;
    }
}

async fn run_app(app: &App) -> Result<(), String> {
    // The tuple literal's element types are not written (`Vec::new()` is generic), yet the tuple
    // *shape* `(_, _)` is what `HandleBranch` destructures — the anchor must recover it.
    app.handle(PhantomData::<Branch<ReadName>>, (Vec::new(), Vec::new()))
        .await?;
    Ok(())
}

fn main() {
    let _ = run_app;
}
