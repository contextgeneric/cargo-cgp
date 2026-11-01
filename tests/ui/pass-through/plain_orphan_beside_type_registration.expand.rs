#![feature(prelude_import)]
//! A plain Rust orphan impl in a crate that also makes legal per-type registrations.
//!
//! An `E0117` names no machinery parameter, so the orphan stage scans the crate's registrations
//! for the impl at its caret. These two are legal, since `DefaultImpls1<ShowImplComponent>` carries
//! a local component, and neither sits at the caret, so the error must stay as rustc wrote it
//! rather than read as `[CGP-E022]` about a key the program registered correctly.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::component::DefaultImpls1;
use cgp::prelude::*;
pub trait Show<T> {
    fn show(&self, value: &T) -> String;
}
impl<__Context__, T> Show<T> for __Context__
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
    __Context__: Show<T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        __Context__::show(__context__, value)
    }
}
impl<__Context__, T> IsProviderFor<ShowImplComponent, __Context__, (T)> for UseContext
where
    __Context__: Show<T>,
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
impl<__Context__> ShowImpl<__Context__, u64> for ShowNumber {
    fn show(__context__: &__Context__, value: &u64) -> String {
        value.to_string()
    }
}
impl<__Context__> IsProviderFor<ShowImplComponent, __Context__, (u64)> for ShowNumber {}
pub struct ShowNumber;
impl<__Components__> DefaultImpls1<ShowImplComponent, __Components__> for u64 {
    type Delegate = ShowNumber;
}
impl std::fmt::Display for Vec<u8> {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
fn main() {}
