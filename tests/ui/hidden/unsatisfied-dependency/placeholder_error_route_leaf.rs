//! A pipeline call whose first stage raises its input's error type, with the input left to inference.
//!
//! `App` runs `PipeHandlers<Product![ParseInput, GreetName]>` and lacks the `name` field `GreetName`
//! reads, the real mistake and the one root cause shown. `ParseInput` requires
//! `CanRaiseError<Input::Error>`, and the call's `Vec::new()` input is not written, so the
//! call-site anchor seeds it as an unknown. The lookup `@ErrorRaiserComponent.<unknown>` would hold
//! for the input's real error type, `String`, so it is not reported as a missing entry.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E105 CGP-E106

use core::marker::PhantomData;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::ReturnError;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;

/// An input that names the error type reading it can produce.
pub trait HasInputError {
    type Error;
}

impl HasInputError for Vec<u8> {
    type Error = String;
}

#[cgp_impl(new ParseInput)]
#[use_type(HasErrorType.Error)]
impl<Code, Input> Handler<Code, Input>
where
    Input: HasInputError + Send,
    Self: CanRaiseError<Input::Error>,
{
    type Output = u64;

    async fn handle(&self, _code: PhantomData<Code>, _input: Input) -> Result<u64, Error> {
        Ok(1)
    }
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetName)]
#[use_type(HasErrorType.Error)]
impl<Code> Handler<Code, u64>
where
    Self: HasName,
{
    type Output = String;

    async fn handle(&self, _code: PhantomData<Code>, _input: u64) -> Result<String, Error> {
        Ok(self.name().to_owned())
    }
}

#[derive(HasField)]
pub struct App {
    pub id: u64,
}

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        @ErrorRaiserComponent.String: ReturnError,
        HandlerComponent: PipeHandlers<Product![ParseInput, GreetName]>,
    }
}

pub async fn run(app: &App) -> Result<String, String> {
    app.handle(PhantomData::<()>, Vec::new()).await
}

fn main() {}
