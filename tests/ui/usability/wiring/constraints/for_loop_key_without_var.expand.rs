#![feature(prelude_import)]
//! A `for` loop whose key does not mention the loop variable.
//!
//! Three `E0207`s with contradictory helps, the shape of `unconstrained_generic`.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;
pub trait CanShow<T> {
    fn show(&self, value: &T) -> String;
}
impl<__Context__, T> CanShow<T> for __Context__
where
    __Context__: ShowImpl<__Context__, T>,
{
    fn show(&self, value: &T) -> String {
        __Context__::show(self, value)
    }
}
pub trait ShowImpl<__Context__, T>: IsProviderFor<ShowImplComponent, __Context__, (T)> {
    fn show(__context__: &__Context__, value: &T) -> String;
}
impl<__Provider__, __Context__, T> ShowImpl<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<ShowImplComponent>
        + IsProviderFor<ShowImplComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        ShowImplComponent,
    >>::Delegate: ShowImpl<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Provider__ as DelegateComponent<
            ShowImplComponent,
        >>::Delegate::show(__context__, value)
    }
}
pub struct ShowImplComponent;
impl<__Context__, T> ShowImpl<__Context__, T> for UseContext
where
    __Context__: CanShow<T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        __Context__::show(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<ShowImplComponent, __Context__, (T)> for UseContext
where
    __Context__: CanShow<T>,
{}
impl<__Context__, T, __Components__, __Path__> ShowImpl<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: ShowImpl<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::show(__context__, value)
    }
}
impl<
    __Context__,
    T,
    __Components__,
    __Path__,
> IsProviderFor<ShowImplComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<ShowImplComponent, __Context__, (T)>
        + ShowImpl<__Context__, T>,
{}
impl<__Components__> DefaultNamespace<__Components__> for ShowImplComponent {
    type Delegate = RedirectLookup<__Components__, Path!(@test.ShowImplComponent)>;
}
impl<__Context__> ShowImpl<__Context__, String> for ShowString {
    fn show(__context__: &__Context__, value: &String) -> String {
        value.clone()
    }
}
impl<__Context__> IsProviderFor<ShowImplComponent, __Context__, (String)>
for ShowString {}
pub struct ShowString;
impl<__Components__> DefaultImpls1<ShowImplComponent, __Components__> for String {
    type Delegate = ShowString;
}
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
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
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
    T,
    Provider,
> DelegateComponent<
    PathCons<
        Symbol!("test"),
        PathCons<ShowImplComponent, PathCons<String, __Wildcard__>>,
    >,
> for App
where
    T: DefaultImpls1<ShowImplComponent, App, Delegate = Provider>,
{
    type Delegate = Provider;
}
impl<
    __Wildcard__,
    T,
    Provider,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<
        Symbol!("test"),
        PathCons<ShowImplComponent, PathCons<String, __Wildcard__>>,
    >,
    __Context__,
    __Params__,
> for App
where
    T: DefaultImpls1<ShowImplComponent, App, Delegate = Provider>,
    Provider: IsProviderFor<
        PathCons<
            Symbol!("test"),
            PathCons<ShowImplComponent, PathCons<String, __Wildcard__>>,
        >,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
