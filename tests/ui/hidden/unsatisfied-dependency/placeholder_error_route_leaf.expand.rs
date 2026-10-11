#![feature(prelude_import)]
//! A pipeline call whose first stage raises its input's error type, with the input left to inference.
//!
//! `App` runs `PipeHandlers<Product![ParseInput, GreetName]>` and lacks the `name` field `GreetName`
//! reads, the real mistake and the one root cause shown. `ParseInput` requires
//! `CanRaiseError<Input::Error>`, and the call's `Vec::new()` input is not written, so the
//! call-site anchor seeds it as an unknown. The lookup `@ErrorRaiserComponent.<unknown>` would hold
//! for the input's real error type, `String`, so it is not reported as a missing entry.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for ParseInput
where
    Input: HasInputError + Send,
    __Context__: CanRaiseError<Input::Error>,
    __Context__: HasErrorType,
{
    type Output = u64;
    async fn handle(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        _input: Input,
    ) -> Result<u64, <__Context__ as HasErrorType>::Error> {
        Ok(1)
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for ParseInput
where
    Input: HasInputError + Send,
    __Context__: CanRaiseError<Input::Error>,
    __Context__: HasErrorType,
{}
pub struct ParseInput;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
impl<__Context__, Code> Handler<__Context__, Code, u64> for GreetName
where
    __Context__: HasName,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        _input: u64,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(__context__.name().to_owned())
    }
}
impl<__Context__, Code> IsProviderFor<HandlerComponent, __Context__, (Code, u64)>
for GreetName
where
    __Context__: HasName,
    __Context__: HasErrorType,
{}
pub struct GreetName;
pub struct App {
    pub id: u64,
}
impl HasField<Symbol!("id")> for App {
    type Value = u64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("id")>,
    ) -> &Self::Value {
        &self.id
    }
}
impl HasFieldMut<Symbol!("id")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("id")>,
    ) -> &mut Self::Value {
        &mut self.id
    }
}
impl DelegateComponent<ErrorRaiserComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ErrorRaiserComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@ErrorRaiserComponent),
    >: IsProviderFor<ErrorRaiserComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>>
for App {
    type Delegate = ReturnError;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    ReturnError: IsProviderFor<
        PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl DelegateComponent<HandlerComponent> for App {
    type Delegate = PipeHandlers<Product![ParseInput, GreetName]>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for App
where
    PipeHandlers<
        Product![ParseInput, GreetName],
    >: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
pub async fn run(app: &App) -> Result<String, String> {
    app.handle(PhantomData::<()>, Vec::new()).await
}
fn main() {}
