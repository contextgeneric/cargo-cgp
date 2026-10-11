#![feature(prelude_import)]
//! `delegate_and_check_components!` with a generic key, `<I> FooKeyComponent<I>: AnyFoo`.
//!
//! The failure is reshaped as `[CGP-E009] the trait `__CanUseApp<FooKeyComponent<I>, ()>``, presenting
//! the generated check trait as a trait the user wrote, over a `[CGP-E107]` leaf.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::prelude::*;
pub trait CanFoo<I> {
    fn foo(&self, tag: PhantomData<I>) -> u8;
}
impl<__Context__, I> CanFoo<I> for __Context__
where
    __Context__: FooProvider<__Context__, I>,
{
    fn foo(&self, tag: PhantomData<I>) -> u8 {
        __Context__::foo(self, tag)
    }
}
pub trait FooProvider<
    __Context__,
    I,
>: IsProviderFor<FooKeyComponent<I>, __Context__, (I)> {
    fn foo(__context__: &__Context__, tag: PhantomData<I>) -> u8;
}
impl<__Provider__, __Context__, I> FooProvider<__Context__, I> for __Provider__
where
    __Provider__: DelegateComponent<FooKeyComponent<I>>
        + IsProviderFor<FooKeyComponent<I>, __Context__, (I)>,
    <__Provider__ as DelegateComponent<
        FooKeyComponent<I>,
    >>::Delegate: FooProvider<__Context__, I>,
{
    fn foo(__context__: &__Context__, tag: PhantomData<I>) -> u8 {
        <__Provider__ as DelegateComponent<
            FooKeyComponent<I>,
        >>::Delegate::foo(__context__, tag)
    }
}
pub struct FooKeyComponent<I>(pub ::core::marker::PhantomData<I>);
impl<__Context__, I> FooProvider<__Context__, I> for UseContext
where
    __Context__: CanFoo<I>,
{
    fn foo(__context__: &__Context__, tag: PhantomData<I>) -> u8 {
        __Context__::foo(__context__, tag)
    }
}
impl<__Context__, I> IsProviderFor<FooKeyComponent<I>, __Context__, (I)> for UseContext
where
    __Context__: CanFoo<I>,
{}
impl<__Context__, I, __Components__, __Path__> FooProvider<__Context__, I>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@I)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@I)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@I)>>::Output,
    >>::Delegate: FooProvider<__Context__, I>,
{
    fn foo(__context__: &__Context__, tag: PhantomData<I>) -> u8 {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@I)>>::Output,
        >>::Delegate::foo(__context__, tag)
    }
}
impl<
    __Context__,
    I,
    __Components__,
    __Path__,
> IsProviderFor<FooKeyComponent<I>, __Context__, (I)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@I)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@I)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@I)>>::Output,
    >>::Delegate: IsProviderFor<FooKeyComponent<I>, __Context__, (I)>
        + FooProvider<__Context__, I>,
{}
impl<__Context__, I> FooProvider<__Context__, I> for AnyFoo {
    fn foo(__context__: &__Context__, _tag: PhantomData<I>) -> u8 {
        0
    }
}
impl<__Context__, I> IsProviderFor<FooKeyComponent<I>, __Context__, (I)> for AnyFoo {}
pub struct AnyFoo;
pub struct App;
impl<I> DelegateComponent<FooKeyComponent<I>> for App {
    type Delegate = AnyFoo;
}
impl<
    I,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<FooKeyComponent<I>, __Context__, __Params__> for App
where
    AnyFoo: IsProviderFor<FooKeyComponent<I>, __Context__, __Params__>,
{}
trait __CanUseApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl<I> __CanUseApp<FooKeyComponent<I>, ()> for App {}
fn main() {}
