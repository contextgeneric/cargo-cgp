#![feature(prelude_import)]
//! A direct entry for `@DescriberComponent.Wrap<u64>` beside a later entry that redirects the
//! overlapping `@DescriberComponent.<T> Wrap<T>`. The two `DelegateComponent<PathCons<..>>` impls
//! conflict with the coherence error E0119 over `Wrap<u64>`.
//!
//! One entry redirects, so the tool reads the pair as a redirect collision on one key and reports
//! `App` wiring `@DescriberComponent.Wrap<T>` directly, which is false: that entry is the redirect.
//!
//! The overlapping counterpart of [open_prefixed_component.rs](open_prefixed_component.rs), whose
//! two entries map one key.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.
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
