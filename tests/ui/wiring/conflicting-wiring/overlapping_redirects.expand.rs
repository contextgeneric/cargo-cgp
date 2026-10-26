#![feature(prelude_import)]
//! Two `delegate_components!` entries that redirect different but overlapping `@`-paths: the
//! generic segment of `@app.<T> T` also matches the `Bar` of `@app.Bar`. The two
//! `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119 over the one key
//! they share.
//!
//! Both entries redirect, so the tool takes the pair for one key redirected twice and reports `App`
//! redirecting `@app.T` twice, which is false: each path is redirected once.
//!
//! The redirect counterpart of
//! [path_generic_type_segment_overlap.rs](path_generic_type_segment_overlap.rs), whose two entries
//! name providers.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Bar;
pub struct App;
impl<
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<Bar, __Wildcard__>>> for App {
    type Delegate = RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<Symbol!("first"), __Wildcard__>>,
    >;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<Bar, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<Symbol!("first"), __Wildcard__>>,
    >: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<Bar, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    T,
    __Wildcard__,
> DelegateComponent<PathCons<Symbol!("app"), PathCons<T, __Wildcard__>>> for App {
    type Delegate = RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<Symbol!("second"), __Wildcard__>>,
    >;
}
impl<
    T,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<Symbol!("app"), PathCons<T, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<Symbol!("second"), __Wildcard__>>,
    >: IsProviderFor<
        PathCons<Symbol!("app"), PathCons<T, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
