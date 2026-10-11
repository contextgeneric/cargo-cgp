#![feature(prelude_import)]
//! A call-site `E0277` whose input writes its type in every shape the call-site anchor reads.
//!
//! The program is `call_site_tuple_input`'s: `Branch<ReadName>` runs `ReadName` on the first
//! component of a tuple input, and `ReadName` reads a `name` field `App` lacks. Here every element
//! of the input writes its type: literals of each definite kind (`"text"`, `1u8`, `true`, `'c'`,
//! `b'a'`, `2.0f32`), a struct literal, a constant, a call to a non-generic function, and a
//! `PhantomData` whose type argument holds a tuple, a reference, a slice, and generic paths. The
//! anchor recovers the whole input type from the call, so the consumer trait in the header names
//! it in full.
//!
//! See cgp-knowledge-base/cgp/errors/hidden/unsatisfied-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
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
    __Params__: ?Sized,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: MyNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
pub struct Marker {
    pub id: u32,
}
pub const LIMIT: u32 = 3;
pub fn make_label() -> String {
    String::new()
}
async fn run_app(app: &App) -> Result<(), String> {
    app.handle(
            PhantomData::<Branch<ReadName>>,
            (
                ("text", 1u8, true, 'c', b'a', 2.0f32),
                (
                    Marker { id: 0 },
                    LIMIT,
                    make_label(),
                    PhantomData::<(u8, &'static [u16], Vec<String>)>,
                ),
            ),
        )
        .await?;
    Ok(())
}
fn main() {
    let _ = run_app;
}
