#![feature(prelude_import)]
//! A direct entry for `@DescriberComponent.Wrap<u64>` beside a later entry that redirects the
//! overlapping `@DescriberComponent.<T> Wrap<T>`. The two `DelegateComponent<PathCons<..>>` impls
//! conflict with the coherence error E0119 over `Wrap<u64>`.
//!
//! The two entries map different keys, so the tool words the pair as an overlap,
//! `[CGP-E011] conflicting wiring: `App` wires `@DescriberComponent.Wrap<u64>`, which
//! `@DescriberComponent.Wrap<T>` already covers`, rather than as a key wired directly beside its
//! own redirect ([open_prefixed_component.rs](open_prefixed_component.rs)).
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanDescribe<T> {
    fn describe(&self, value: &T) -> String;
}
impl<__Context__, T> CanDescribe<T> for __Context__
where
    __Context__: Describer<__Context__, T>,
{
    fn describe(&self, value: &T) -> String {
        __Context__::describe(self, value)
    }
}
pub trait Describer<
    __Context__,
    T,
>: IsProviderFor<DescriberComponent, __Context__, (T)> {
    fn describe(__context__: &__Context__, value: &T) -> String;
}
impl<__Provider__, __Context__, T> Describer<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<DescriberComponent>
        + IsProviderFor<DescriberComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        DescriberComponent,
    >>::Delegate: Describer<__Context__, T>,
{
    fn describe(__context__: &__Context__, value: &T) -> String {
        <__Provider__ as DelegateComponent<
            DescriberComponent,
        >>::Delegate::describe(__context__, value)
    }
}
///Component marker for [`CanDescribe`].
pub struct DescriberComponent;
impl<__Context__, T> Describer<__Context__, T> for UseContext
where
    __Context__: CanDescribe<T>,
{
    fn describe(__context__: &__Context__, value: &T) -> String {
        __Context__::describe(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<DescriberComponent, __Context__, (T)> for UseContext
where
    __Context__: CanDescribe<T>,
{}
impl<__Context__, T, __Components__, __Path__> Describer<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: Describer<__Context__, T>,
{
    fn describe(__context__: &__Context__, value: &T) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::describe(__context__, value)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<DescriberComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<DescriberComponent, __Context__, (T)>
        + Describer<__Context__, T>,
{}
impl<__Context__, T> Describer<__Context__, T> for DescribeAny {
    fn describe(__context__: &__Context__, _value: &T) -> String {
        "a value".to_owned()
    }
}
impl<__Context__, T> IsProviderFor<DescriberComponent, __Context__, (T)>
for DescribeAny {}
///Provider `DescribeAny` for [`Describer`].
pub struct DescribeAny;
pub struct Wrap<T>(pub T);
pub struct App;
impl<
    __Wildcard__,
> DelegateComponent<PathCons<DescriberComponent, PathCons<Wrap<u64>, __Wildcard__>>>
for App {
    type Delegate = DescribeAny;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<DescriberComponent, PathCons<Wrap<u64>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    DescribeAny: IsProviderFor<
        PathCons<DescriberComponent, PathCons<Wrap<u64>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    T,
    __Wildcard__,
> DelegateComponent<PathCons<DescriberComponent, PathCons<Wrap<T>, __Wildcard__>>>
for App {
    type Delegate = RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<Symbol!("describer"), __Wildcard__>>,
    >;
}
impl<
    T,
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<DescriberComponent, PathCons<Wrap<T>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    RedirectLookup<
        App,
        PathCons<Symbol!("app"), PathCons<Symbol!("describer"), __Wildcard__>>,
    >: IsProviderFor<
        PathCons<DescriberComponent, PathCons<Wrap<T>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
