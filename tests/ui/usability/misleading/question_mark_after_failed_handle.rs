//! A `BuildAndMergeOutputs` builder whose second output carries a field the target lacks, called with
//! `?` and no check.
//!
//! The only error is `[CGP-E001] the consumer trait `CanRaiseError<()>` is not implemented` with root
//! cause `(): Error`, a trait and a cause unrelated to the mistake; the missing `url` field on `App`
//! is never mentioned. Distilled from the application builder with its `http_client` field removed.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::dispatch::BuildAndMergeOutputs;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent};
use cgp::prelude::*;

mod backend {
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

#[derive(CgpData)]
pub struct Db {
    pub db_path: String,
}

#[derive(CgpData)]
pub struct Http {
    pub url: String,
}

// `App` lacks the `url` field that `BuildHttp`'s output carries.
#[derive(CgpData)]
pub struct App {
    pub db_path: String,
}

#[cgp_impl(new BuildDb)]
#[use_type(HasErrorType.Error)]
impl<Code, Input> Handler<Code, Input> {
    type Output = Db;

    async fn handle(&self, _code: PhantomData<Code>, _input: Input) -> Result<Db, Error> {
        Ok(Db { db_path: "db".to_owned() })
    }
}

#[cgp_impl(new BuildHttp)]
#[use_type(HasErrorType.Error)]
impl<Code, Input> Handler<Code, Input> {
    type Output = Http;

    async fn handle(&self, _code: PhantomData<Code>, _input: Input) -> Result<Http, Error> {
        Ok(Http { url: "url".to_owned() })
    }
}

pub struct AppBuilder;

delegate_components! {
    AppBuilder {
        ErrorTypeProviderComponent: UseType<backend::Error>,
        ErrorRaiserComponent: backend::RaiseBackendError,
        HandlerComponent: BuildAndMergeOutputs<App, Product![BuildDb, BuildHttp]>,
    }
}

pub async fn run() -> Result<(), backend::Error> {
    let _app = AppBuilder.handle(PhantomData::<()>, ()).await?;
    Ok(())
}

fn main() {}
