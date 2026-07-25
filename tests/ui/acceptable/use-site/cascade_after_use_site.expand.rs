#![feature(prelude_import)]
//! A use-site `E0277` on a `Code`-dispatched handler pipeline, resolved from the
//! call expression itself.
//!
//! `App` runs a program through the async, `Code`-dispatched `Handler` component,
//! dispatched to a `PipeHandlers` composition (the same combinator plumbing a real
//! DSL context produces). The first stage, `HandleName`, reads the context's `name`
//! field, but `App` has none, so awaiting `app.handle(..)` at a use site fails.
//! Because the composition matches the `Code` unconditionally the method is *found*
//! (its trait bound is only conditionally satisfied), so the failure is an `E0277`,
//! not an `E0599`, and its spans never land on `App`'s type definition — no
//! span-matching anchor applies. The call-site anchor recovers it from the failing
//! call instead: the context from the receiver's binding, the `Code` from the
//! `PhantomData` tag argument the method's own signature declares, and the inferred
//! input as an unknown the walk resolves around — reaching the missing `name` field
//! and rendering the full combinator chain, with the header naming the consumer
//! trait the call needs rather than the `PipeHandlers`/`ComposeHandlers` plumbing.
//!
//! One block remains of what used to be four: the re-report rustc raises where the
//! result is awaited resolves to the same cause and de-duplicates away, and the
//! trailing `?`-operator `Try`/`FromResidual` cascade — which restates the failure
//! and dumps the unresolved `<App as CanHandle<..>>::Output` projection — is
//! suppressed by the span-overlap gate.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent, PipeHandlers};
use cgp::prelude::*;
/// A program wrapper generic over its pipeline steps — the analogue of the DSL's
/// `Pipe<Steps>`, wired to a single provider that matches *every* `Prog<Steps>`.
pub struct Prog<Steps>(pub PhantomData<Steps>);
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
/// First pipe stage: produce the context's name. Depends on `Self: HasName`, which
/// `App` cannot meet — the buried root cause.
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for HandleName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Code>,
        _input: Input,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(__context__.name().to_owned())
    }
}
impl<
    __Context__,
    Code,
    Input,
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for HandleName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandleName;
/// Second pipe stage: uppercase the name. Takes the first stage's `String` output.
impl<__Context__, Code> Handler<__Context__, Code, String> for HandleShout
where
    __Context__: Sync,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Code>,
        input: String,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(input.to_uppercase())
    }
}
impl<__Context__, Code> IsProviderFor<HandlerComponent, __Context__, (Code, String)>
for HandleShout
where
    __Context__: Sync,
    __Context__: HasErrorType,
{}
pub struct HandleShout;
pub struct __MyNamespaceComponents;
pub trait MyNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> MyNamespace<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__MyNamespaceComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("core"),
        PathCons<Symbol!("error"), PathCons<ErrorTypeProviderComponent, __Wildcard__>>,
    >,
> {
    type Delegate = UseType<String>;
}
impl<__Table__, Steps, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Prog<Steps>, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = PipeHandlers<Steps>;
}
pub struct App {}
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: MyNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: MyNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Prog<Product![HandleName, HandleShout]>>, Vec::new())
        .await?;
    Ok(())
}
fn main() {
    let _ = run_app;
}
