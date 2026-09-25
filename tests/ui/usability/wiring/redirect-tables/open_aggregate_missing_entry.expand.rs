#![feature(prelude_import)]
//! Usability failure: a missing entry in an aggregate provider's own `open` table is reported as if
//! it were missing from the context. The context `App` opens `ComputerComponent` and routes the
//! `Sink` code to the aggregate provider `ByteSink`, which opens `ComputerComponent` in its own
//! table and dispatches on the *input*, the component's second parameter, with a two-segment
//! `@ComputerComponent.<Code> Code.Bytes` key. `ByteSink` has no entry for the `Digest` input the
//! check asks for, so the real root cause is the unmet
//! `ByteSink: DelegateComponent<@ComputerComponent.Sink.Digest>`.
//!
//! The root cause is present, and the path is right, but two labels misname the table. The second
//! `[CGP-E104]` redirect hop, the one that runs inside `ByteSink` (`RedirectLookup<ByteSink, …>`),
//! is rendered `redirect lookup to @ComputerComponent in App`, because the hop takes the walk's
//! context rather than the redirect's own table. And the `[CGP-E107]` leaf names the right owner
//! but calls it a context: `context ByteSink does not contain any delegate entry for …`, where a
//! provider table missing a key is otherwise the `[CGP-E110]` "provider … does not contain" leaf.
//! A reader cannot tell from the tree which table to add the entry to without knowing that the
//! second hop is the aggregate's.
//!
//! See cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;
/// The code that routes a computation to the sink aggregate.
pub struct Sink;
/// An input the sink handles, and one it does not.
pub struct Bytes(pub Vec<u8>);
pub struct Digest(pub [u8; 4]);
impl<__Context__, Code> Computer<__Context__, Code, Bytes> for WriteBytes {
    type Output = usize;
    fn compute(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        input: Bytes,
    ) -> usize {
        input.0.len()
    }
}
impl<__Context__, Code> IsProviderFor<ComputerComponent, __Context__, (Code, Bytes)>
for WriteBytes {}
pub struct WriteBytes;
pub struct ByteSink;
impl DelegateComponent<ComputerComponent> for ByteSink {
    type Delegate = RedirectLookup<ByteSink, Path!(@ComputerComponent)>;
}
impl<__Context__, __Params__> IsProviderFor<ComputerComponent, __Context__, __Params__>
for ByteSink
where
    RedirectLookup<
        ByteSink,
        Path!(@ComputerComponent),
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
impl<
    Code,
    __Wildcard__,
> DelegateComponent<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Bytes, __Wildcard__>>>,
> for ByteSink {
    type Delegate = WriteBytes;
}
impl<
    Code,
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Code, PathCons<Bytes, __Wildcard__>>>,
    __Context__,
    __Params__,
> for ByteSink
where
    WriteBytes: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Code, PathCons<Bytes, __Wildcard__>>>,
        __Context__,
        __Params__,
    >,
{}
pub struct App;
impl DelegateComponent<ComputerComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ComputerComponent)>;
}
impl<__Context__, __Params__> IsProviderFor<ComputerComponent, __Context__, __Params__>
for App
where
    RedirectLookup<
        App,
        Path!(@ComputerComponent),
    >: IsProviderFor<ComputerComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ComputerComponent, PathCons<Sink, __Wildcard__>>> for App {
    type Delegate = ByteSink;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Sink, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    ByteSink: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Sink, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ComputerComponent, (Sink, Digest)> for App {}
fn main() {}
