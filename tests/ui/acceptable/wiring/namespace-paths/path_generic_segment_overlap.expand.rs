#![feature(prelude_import)]
//! Acceptable failure: an `open` key with a generic first segment overlapping a one-segment key.
//! `@ComputerComponent.<Code> Code.u64` dispatches on the input, the component's second parameter,
//! for every code, so at `Code = Eval` it covers a key that `@ComputerComponent.Eval` already
//! claims. Each path entry lowers to a `DelegateComponent<PathCons<..>>` with a wildcard tail, and
//! the generic segment makes the two impls overlap with the coherence error E0119.
//!
//! The tool drops the redundant `IsProviderFor` half and rewrites the `DelegateComponent` half to
//! `[CGP-E005] `App` cannot wire `@ComputerComponent.*.u64.*` that is already set through
//! `@ComputerComponent.Eval.*``: the generic segment renders as `*` in its own position, so the
//! concrete `u64` segment after it still shows which key the entry names.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E005).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;
/// The code the one-segment key names.
pub struct Eval;
impl<__Context__, Code> Computer<__Context__, Code, u64> for Double {
    type Output = u64;
    fn compute(__context__: &__Context__, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}
impl<__Context__, Code> IsProviderFor<ComputerComponent, __Context__, (Code, u64)>
for Double {}
pub struct Double;
impl<__Context__, Code, Input: core::fmt::Debug> Computer<__Context__, Code, Input>
for Show {
    type Output = String;
    fn compute(
        __context__: &__Context__,
        _code: PhantomData<Code>,
        input: Input,
    ) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0:?}", input))
        })
    }
}
impl<
    __Context__,
    Code,
    Input: core::fmt::Debug,
> IsProviderFor<ComputerComponent, __Context__, (Code, Input)> for Show {}
pub struct Show;
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
> DelegateComponent<PathCons<ComputerComponent, PathCons<Eval, __Wildcard__>>> for App {
    type Delegate = Show;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Eval, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    Show: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Eval, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    Code,
    __Wildcard__,
> DelegateComponent<
    PathCons<ComputerComponent, PathCons<Code, PathCons<u64, __Wildcard__>>>,
> for App {
    type Delegate = Double;
}
impl<
    Code,
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<ComputerComponent, PathCons<Code, PathCons<u64, __Wildcard__>>>,
    __Context__,
    __Params__,
> for App
where
    Double: IsProviderFor<
        PathCons<ComputerComponent, PathCons<Code, PathCons<u64, __Wildcard__>>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
