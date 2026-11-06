#![feature(prelude_import)]
//! A method of a consumer trait with a parameter, called on an unbounded type parameter.
//!
//! `fn run<Context>` calls `context.show(1u32)` without bounding `Context` by `CanShow<T>`, whose
//! parameter the bound would have to name. The call writes `u32`, but a bound written from a call's
//! arguments would be a guess, so the tool names no bound and keeps rustc's error.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanShow<T> {
    fn show(&self, value: T);
}
impl<__Context__, T> CanShow<T> for __Context__
where
    __Context__: Shower<__Context__, T>,
{
    fn show(&self, value: T) {
        __Context__::show(self, value)
    }
}
pub trait Shower<__Context__, T>: IsProviderFor<ShowerComponent, __Context__, (T)> {
    fn show(__context__: &__Context__, value: T);
}
impl<__Provider__, __Context__, T> Shower<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<ShowerComponent>
        + IsProviderFor<ShowerComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        ShowerComponent,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: T) {
        <__Provider__ as DelegateComponent<
            ShowerComponent,
        >>::Delegate::show(__context__, value)
    }
}
pub struct ShowerComponent;
impl<__Context__, T> Shower<__Context__, T> for UseContext
where
    __Context__: CanShow<T>,
{
    fn show(__context__: &__Context__, value: T) {
        __Context__::show(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<ShowerComponent, __Context__, (T)> for UseContext
where
    __Context__: CanShow<T>,
{}
impl<__Context__, T, __Components__, __Path__> Shower<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: T) {
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
> IsProviderFor<ShowerComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<ShowerComponent, __Context__, (T)>
        + Shower<__Context__, T>,
{}
fn run<Context>(context: &Context) {
    context.show(1u32);
}
fn main() {}
