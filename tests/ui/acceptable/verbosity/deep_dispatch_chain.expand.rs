#![feature(prelude_import)]
//! A resolved dispatch chain over a DSL-sized program, every construct named in full.
//!
//! `App` runs a four-stage pipeline program through the `Code`-dispatched `Handler`
//! component, inherited through two namespace levels. The first stage reads the
//! context's `name` field, which `App` lacks, and the call-site anchor resolves the
//! failure cleanly — root cause first, one block. What this fixture pins is the
//! *presentation* of that success on a program-sized `Code` type: the whole chain
//! unfolds stage by stage down to the missing field, and each hop states its trait and
//! parameters as written.
//!
//! A hop repeating its parent's trait exactly once rendered as `Handler<…>` to keep the
//! chain short. That elision was removed deliberately: it hid the very type a reader
//! follows the chain to trace, and made a genuine repeat indistinguishable from a hop
//! whose parameters differ. The length here is the accepted cost, and the snapshot is
//! what makes it visible when it changes.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp/blob/main/docs/errors/checks/check-trait-failure.md (use-site face).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
/// `App` cannot meet — the root cause.
impl<__Context__, Code, Input> Handler<__Context__, Code, Input> for HandleReadName
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
> IsProviderFor<HandlerComponent, __Context__, (Code, Input)> for HandleReadName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandleReadName;
/// A pass-through stage, generic over a style marker, so the pipeline has depth and
/// each step a distinct, DSL-sized name.
impl<__Context__, Code, Style> Handler<__Context__, Code, String> for HandlePass<Style>
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
        Ok(input)
    }
}
impl<
    __Context__,
    Code,
    Style,
> IsProviderFor<HandlerComponent, __Context__, (Code, String)> for HandlePass<Style>
where
    __Context__: Sync,
    __Context__: HasErrorType,
{}
pub struct HandlePass<Style>(pub ::core::marker::PhantomData<Style>);
pub struct __BaseNamespaceComponents;
pub trait BaseNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> BaseNamespace<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__BaseNamespaceComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<__Table__, __Wildcard__> BaseNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("core"),
        PathCons<Symbol!("error"), PathCons<ErrorTypeProviderComponent, __Wildcard__>>,
    >,
> {
    type Delegate = UseType<String>;
}
impl<__Table__, Steps, __Wildcard__> BaseNamespace<__Table__>
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
pub struct __ExtendedNamespaceComponents;
pub trait ExtendedNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> ExtendedNamespace<__Table__> for __Key__
where
    __Key__: BaseNamespace<__ExtendedNamespaceComponents>,
    __Key__: BaseNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
pub struct App {}
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: ExtendedNamespace<App, Delegate = __Value__>,
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
    __Key__: ExtendedNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
pub type Program = Prog<
    Product![
        HandleReadName, HandlePass<Politely>, HandlePass<Shout>, HandlePass<
        Repeat>
    ],
>;
async fn run_app(app: &App) -> Result<(), String> {
    app.handle(PhantomData::<Program>, Vec::new()).await?;
    Ok(())
}
fn main() {
    let _ = run_app;
}
