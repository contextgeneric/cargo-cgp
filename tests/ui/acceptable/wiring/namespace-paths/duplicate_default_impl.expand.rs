#![feature(prelude_import)]
//! Acceptable failure: two `#[cgp_impl]` blocks each carrying a
//! `#[default_impl(String in DefaultImpls1<..>)]` for the same key emit two
//! conflicting `DefaultImpls1<ShowImplComponent, __Components__>` impls for
//! `String`, which the Rust compiler rejects with the coherence error E0119.
//! `#[cgp_impl]` lowers each block independently and has no view of the other,
//! so it correctly defers to the compiler, exactly as two hand-written
//! overlapping impls would.
//!
//! The carets fall on the `String` key inside `#[default_impl(...)]` rather than
//! on the whole `#[cgp_impl]` attribute, because the synthesized default-impl is
//! re-spanned onto that key token (see
//! cgp-macro-core/src/types/attributes/default_impl/attribute.rs). A regression
//! that dropped the re-span would move the carets back onto the macro attribute.
//!
//! See docs/errors/wiring/conflicting-wiring.md.
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
impl<__Context__> ShowImpl<__Context__, String> for ShowStringA {
    fn show(__context__: &__Context__, value: &String) -> String {
        value.clone()
    }
}
impl<__Context__> IsProviderFor<ShowImplComponent, __Context__, (String)>
for ShowStringA {}
pub struct ShowStringA;
impl<__Components__> DefaultImpls1<ShowImplComponent, __Components__> for String {
    type Delegate = ShowStringA;
}
impl<__Context__> ShowImpl<__Context__, String> for ShowStringB {
    fn show(__context__: &__Context__, value: &String) -> String {
        value.clone()
    }
}
impl<__Context__> IsProviderFor<ShowImplComponent, __Context__, (String)>
for ShowStringB {}
pub struct ShowStringB;
impl<__Components__> DefaultImpls1<ShowImplComponent, __Components__> for String {
    type Delegate = ShowStringB;
}
fn main() {}
