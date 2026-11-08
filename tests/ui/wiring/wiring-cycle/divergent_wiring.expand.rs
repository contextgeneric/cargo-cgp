#![feature(prelude_import)]
//! A wiring whose lookup grows without repeating: `ShowNested` implements `Shower<T>` by requiring
//! `CanShow<Vec<T>>` of the context, so checking `CanShow<u32>` asks for `CanShow<Vec<u32>>`, then
//! `CanShow<Vec<Vec<u32>>>`, and so on. The trait solver overflows (`E0275`). No obligation
//! repeats, so the resolver's cycle guard never fires; its walk stops at the depth cap, and the tool
//! words the overflow as a lookup that grows (`[CGP-E021]`), naming the first larger requirement,
//! and a help naming `ShowNested` as the provider whose requirement grows.
//!
//! rustc's advice to raise the recursion limit is dropped, since no limit ends this lookup; the
//! output is `acceptable`.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the depth cap is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.
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
///Component marker for [`CanShow`].
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
impl<__Context__, T> Shower<__Context__, T> for ShowNested
where
    __Context__: CanShow<Vec<T>>,
{
    fn show(__context__: &__Context__, value: T) {
        __context__
            .show(
                ::alloc::boxed::box_assume_init_into_vec_unsafe(
                    ::alloc::intrinsics::write_box_via_move(
                        ::alloc::boxed::Box::new_uninit(),
                        [value],
                    ),
                ),
            );
    }
}
impl<__Context__, T> IsProviderFor<ShowerComponent, __Context__, (T)> for ShowNested
where
    __Context__: CanShow<Vec<T>>,
{}
///Provider `ShowNested` for [`Shower`].
pub struct ShowNested;
pub struct App;
impl DelegateComponent<ShowerComponent> for App {
    type Delegate = ShowNested;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ShowerComponent, __Context__, __Params__> for App
where
    ShowNested: IsProviderFor<ShowerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ShowerComponent, u32> for App {}
fn main() {}
