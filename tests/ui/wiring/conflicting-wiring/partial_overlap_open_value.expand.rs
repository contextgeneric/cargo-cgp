#![feature(prelude_import)]
//! Two `@`-path keys that overlap in part, where the key they share keeps a type the keys leave
//! open: `@app.<X> Wrap<X>` and `@app.<T> T.Bar` both map `@app.Wrap<X>.Bar` for every `X`, so the
//! two `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119.
//!
//! Neither key covers the other, so the tool names the key they share, its open type written `_`:
//! `[CGP-E011] conflicting wiring: `App` wires `@app.Wrap<_>.Bar`, which `@app.Wrap<X>` already
//! covers`, as rustc writes that type in its own message.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Wrap<T>(pub T);
pub struct Bar;
pub struct App;
pub struct First;
pub struct Second;
impl<
    X,
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<Wrap<X>, __Wildcard__>>> for App {
    type Delegate = First;
}
impl<
    X,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<Wrap<X>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    First: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<Wrap<X>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    T,
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<T, PathCons<Bar, __Wildcard__>>>>
for App {
    type Delegate = Second;
}
impl<
    T,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<T, PathCons<Bar, __Wildcard__>>>,
    __Context__,
    __Params__,
> for App
where
    Second: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<T, PathCons<Bar, __Wildcard__>>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
