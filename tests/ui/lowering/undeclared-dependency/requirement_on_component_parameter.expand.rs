#![feature(prelude_import)]
//! A provider of a component that requires `HasName` of every context (`#[extend(HasName)]`) and
//! `T: Display` of its parameter, declaring neither.
//!
//! The tool reports `[CGP-E036]` for the context's requirement alone, `HasName`, and leaves rustc's
//! `E0277` on `T: Display`, which `#[uses]` does not declare.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::fmt::Display;
use cgp::prelude::*;
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        ::core::convert::AsRef::as_ref(
            HasField::get_field(self, ::core::marker::PhantomData::<Symbol!("name")>),
        )
    }
}
pub trait CanShow<T: Display>: HasName {
    fn show(&self, value: &T) -> String;
}
impl<__Context__, T: Display> CanShow<T> for __Context__
where
    __Context__: HasName,
    __Context__: Shower<__Context__, T>,
{
    fn show(&self, value: &T) -> String {
        __Context__::show(self, value)
    }
}
pub trait Shower<
    __Context__,
    T: Display,
>: IsProviderFor<ShowerComponent, __Context__, (T)>
where
    __Context__: HasName,
{
    fn show(__context__: &__Context__, value: &T) -> String;
}
impl<__Provider__, __Context__, T: Display> Shower<__Context__, T> for __Provider__
where
    __Context__: HasName,
    __Provider__: DelegateComponent<ShowerComponent>
        + IsProviderFor<ShowerComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        ShowerComponent,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Provider__ as DelegateComponent<
            ShowerComponent,
        >>::Delegate::show(__context__, value)
    }
}
///Component marker for [`CanShow`].
pub struct ShowerComponent;
impl<__Context__, T: Display> Shower<__Context__, T> for UseContext
where
    __Context__: HasName,
    __Context__: CanShow<T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        __Context__::show(__context__, value)
    }
}
impl<__Context__, T: Display> IsProviderFor<ShowerComponent, __Context__, (T)>
for UseContext
where
    __Context__: HasName,
    __Context__: CanShow<T>,
{}
impl<__Context__, T: Display, __Components__, __Path__> Shower<__Context__, T>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasName,
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: &T) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@T)>>::Output,
        >>::Delegate::show(__context__, value)
    }
}
impl<
    __Context__,
    T: Display,
    __Components__,
    __Path__,
> IsProviderFor<ShowerComponent, __Context__, (T)>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasName,
    __Path__: ConcatPath<Path!(@T)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@T)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@T)>>::Output,
    >>::Delegate: IsProviderFor<ShowerComponent, __Context__, (T)>
        + Shower<__Context__, T>,
{}
impl<__Context__, T> Shower<__Context__, T> for ShowPlain {
    fn show(__context__: &__Context__, _value: &T) -> String {
        String::new()
    }
}
impl<__Context__, T> IsProviderFor<ShowerComponent, __Context__, (T)> for ShowPlain {}
///Provider `ShowPlain` for [`Shower`].
pub struct ShowPlain;
fn main() {}
