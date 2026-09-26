#![feature(prelude_import)]
//! Acceptable failure: a missing entry in an aggregate provider's own `open` table. The context
//! `App` opens `ComputerComponent` and routes the `Sink` code to the aggregate provider `ByteSink`,
//! which opens `ComputerComponent` in its own table and dispatches on the *input*, the component's
//! second parameter, with a two-segment `@ComputerComponent.<Code> Code.Bytes` key. `ByteSink` has no
//! entry for the `Digest` input the check asks for, so the root cause is the unmet
//! `ByteSink: DelegateComponent<@ComputerComponent.Sink.Digest>`.
//!
//! The tree names the table each lookup runs in. The first `[CGP-E104]` hop is the context's own
//! redirect, `redirect lookup to @ComputerComponent in App`; the second runs inside the aggregate
//! (`RedirectLookup<ByteSink, …>`) and reads `… in ByteSink`. The leaf is the `[CGP-E110]`
//! provider-table leaf, `provider ByteSink does not contain any delegate entry for
//! @ComputerComponent.Sink.Digest`, so the reader adds the entry to `ByteSink` rather than to `App`.
//!
//! See cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E104, CGP-E110).
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
