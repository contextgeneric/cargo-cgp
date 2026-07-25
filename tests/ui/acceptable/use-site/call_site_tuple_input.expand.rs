#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::{CanHandle, Handler, HandlerComponent};
use cgp::prelude::*;
/// A conditional program that runs its inner `Cond` on the first component of a tuple input.
pub struct Branch<Cond>(pub PhantomData<Cond>);
/// Inner program marker: read the context's `name`.
pub struct ReadName;
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
/// Interpreter for `ReadName`: reads the context's `name` field — the buried root cause.
impl<__Context__, Input> Handler<__Context__, ReadName, Input> for HandleReadName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<ReadName>,
        _input: Input,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        Ok(__context__.name().to_owned())
    }
}
impl<__Context__, Input> IsProviderFor<HandlerComponent, __Context__, (ReadName, Input)>
for HandleReadName
where
    __Context__: HasName,
    Input: Send,
    __Context__: HasErrorType,
{}
pub struct HandleReadName;
/// Interpreter for `Branch<Cond>`: destructures its input as a 2-tuple and runs `Cond` on the first
/// component. Its input *shape* is a tuple, so a call that does not type that tuple must still have
/// its structure recovered for this impl to match.
impl<
    __Context__,
    Cond,
    CondInput,
    Rest,
> Handler<__Context__, Branch<Cond>, (CondInput, Rest)> for HandleBranch
where
    __Context__: CanHandle<Cond, CondInput, Output = String>,
    Rest: Send,
    __Context__: HasErrorType,
{
    type Output = String;
    async fn handle(
        __context__: &__Context__,
        _tag: PhantomData<Branch<Cond>>,
        (cond_input, _rest): (CondInput, Rest),
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        __context__.handle(PhantomData::<Cond>, cond_input).await
    }
}
impl<
    __Context__,
    Cond,
    CondInput,
    Rest,
> IsProviderFor<HandlerComponent, __Context__, (Branch<Cond>, (CondInput, Rest))>
for HandleBranch
where
    __Context__: CanHandle<Cond, CondInput, Output = String>,
    Rest: Send,
    __Context__: HasErrorType,
{}
pub struct HandleBranch;
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
impl<__Table__, Cond, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<Branch<Cond>, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = HandleBranch;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("extra"),
        PathCons<
            Symbol!("handler"),
            PathCons<HandlerComponent, PathCons<ReadName, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = HandleReadName;
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
    app.handle(PhantomData::<Branch<ReadName>>, (Vec::new(), Vec::new())).await?;
    Ok(())
}
fn main() {
    let _ = run_app;
}
