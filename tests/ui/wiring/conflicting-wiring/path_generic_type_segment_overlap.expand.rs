#![feature(prelude_import)]
//! An `open` key whose segment is a generic type, overlapping a key for one of its instances.
//! `@DescriberComponent.<T> Wrap<T>` covers every `Wrap<…>`, so it overlaps the
//! `@DescriberComponent.Wrap<u64>` entry beside it, and the two `DelegateComponent<PathCons<..>>`
//! impls conflict with the coherence error E0119.
//!
//! The tool rewrites the conflict to
//! `[CGP-E011] conflicting wiring: `App` wires `@DescriberComponent.Wrap<u64>`, which `@DescriberComponent.Wrap<T>` already covers`:
//! each type segment keeps its arguments, which tell the two keys apart. Dropping them would render
//! both keys the same and misreport the overlap as a duplicate.
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
pub struct DescribeAny;
pub struct Wrap<T>(pub T);
pub struct App;
impl DelegateComponent<DescriberComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@DescriberComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<DescriberComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@DescriberComponent),
    >: IsProviderFor<DescriberComponent, __Context__, __Params__>,
{}
impl<
    T,
    __Wildcard__,
> DelegateComponent<PathCons<DescriberComponent, PathCons<Wrap<T>, __Wildcard__>>>
for App {
    type Delegate = DescribeAny;
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
    DescribeAny: IsProviderFor<
        PathCons<DescriberComponent, PathCons<Wrap<T>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
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
fn main() {}
