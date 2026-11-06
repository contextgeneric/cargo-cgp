//! Upstream crate for a fixture that runs a handler pipeline on a context defined here.
//!
//! The crate holds a small type-level pipeline language (`Pipe`, `Echo`, `Stdout`), the
//! providers that interpret it, and a context `App` whose wiring routes every syntax but
//! leaves one error type, `IoFailure`, without a raiser. The fixture calls `App` from
//! another crate, which is the shape where the missing route never reaches the output.
//!
//! A fixture pulls it in with a `//@aux-build: cgp-test-foreign-pipeline` header
//! directive.

use cgp::core::component::UseDelegate;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;

pub mod backend {
    use core::error::Error as StdError;

    use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent};
    use cgp::prelude::*;

    #[derive(Debug)]
    pub struct Error(pub String);

    pub struct RaiseBackendError;

    #[cgp_impl(RaiseBackendError)]
    #[use_type(HasErrorType.{Error = crate::backend::Error})]
    impl<E> ErrorRaiser<E>
    where
        E: StdError + Send + Sync + 'static,
    {
        fn raise_error(e: E) -> Error {
            crate::backend::Error(e.to_string())
        }
    }
}

#[derive(Debug)]
pub struct IoFailure;

impl core::fmt::Display for IoFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("io failure")
    }
}

impl core::error::Error for IoFailure {}

#[derive(Debug)]
pub struct ParseFailure;

impl core::fmt::Display for ParseFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("parse failure")
    }
}

impl core::error::Error for ParseFailure {}

// The syntax: a pipeline of stages, each an empty marker.
pub struct Pipe<Handlers>(pub PhantomData<Handlers>);
pub struct Echo;
pub struct Stdout;

// Wraps each stage so the pipeline calls back into the context for it.
pub trait WrapCall {
    type Wrapped;
}

impl WrapCall for Nil {
    type Wrapped = Nil;
}

impl<Head, Tail: WrapCall> WrapCall for Cons<Head, Tail> {
    type Wrapped = Cons<Call<Head>, Tail::Wrapped>;
}

#[cgp_impl(new Call<InCode>)]
impl<OutCode, InCode, Input, Output> Handler<OutCode, Input>
where
    Self: CanHandle<InCode, Input, Output = Output>,
{
    type Output = Output;

    async fn handle(
        &self,
        _tag: PhantomData<OutCode>,
        input: Input,
    ) -> Result<Output, Self::Error> {
        self.handle(PhantomData, input).await
    }
}

delegate_components! {
    new HandlePipe {
        HandlerComponent: UseDelegate<new RunPipeHandler {
            <Handlers: WrapCall> Pipe<Handlers>:
                PipeHandlers<Handlers::Wrapped>,
        }>,
    }
}

#[cgp_impl(new HandleEcho)]
#[uses(CanRaiseError<IoFailure>)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<Echo, Input>
where
    Input: Send + AsRef<[u8]>,
{
    type Output = Vec<u8>;

    async fn handle(&self, _code: PhantomData<Echo>, input: Input) -> Result<Vec<u8>, Error> {
        if input.as_ref().is_empty() {
            return Err(Self::raise_error(IoFailure));
        }
        Ok(input.as_ref().to_vec())
    }
}

#[cgp_impl(new HandleStdout)]
#[use_type(HasErrorType.Error)]
impl<Input> Handler<Stdout, Input>
where
    Input: Send + AsRef<[u8]>,
{
    type Output = ();

    async fn handle(&self, _code: PhantomData<Stdout>, _input: Input) -> Result<(), Error> {
        Ok(())
    }
}

pub struct App;

// The raiser routes `ParseFailure` but not `IoFailure`, which `HandleEcho` raises.
delegate_components! {
    App {
        open {HandlerComponent, ErrorRaiserComponent};

        ErrorTypeProviderComponent: UseType<backend::Error>,
        @ErrorRaiserComponent.ParseFailure: backend::RaiseBackendError,
        @HandlerComponent.<Handlers> Pipe<Handlers>: HandlePipe,
        @HandlerComponent.Echo: HandleEcho,
        @HandlerComponent.Stdout: HandleStdout,
    }
}
