#![feature(prelude_import)]
//! The same entry of a generic component listed twice in one `check_components!` table.
//!
//! The two check impls the table generates for `ShowerComponent: u32` are identical, so the
//! compiler rejects them with an `E0119`, which becomes `[CGP-E015]` naming the entry as written.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanShow<T> {
    fn show(&self, value: T) -> String;
}
impl<__Context__, T> CanShow<T> for __Context__
where
    __Context__: Shower<__Context__, T>,
{
    fn show(&self, value: T) -> String {
        __Context__::show(self, value)
    }
}
pub trait Shower<__Context__, T>: IsProviderFor<ShowerComponent, __Context__, (T)> {
    fn show(__context__: &__Context__, value: T) -> String;
}
impl<__Provider__, __Context__, T> Shower<__Context__, T> for __Provider__
where
    __Provider__: DelegateComponent<ShowerComponent>
        + IsProviderFor<ShowerComponent, __Context__, (T)>,
    <__Provider__ as DelegateComponent<
        ShowerComponent,
    >>::Delegate: Shower<__Context__, T>,
{
    fn show(__context__: &__Context__, value: T) -> String {
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
    fn show(__context__: &__Context__, value: T) -> String {
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
    fn show(__context__: &__Context__, value: T) -> String {
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
impl<__Context__, T: core::fmt::Debug> Shower<__Context__, T> for ShowDebug {
    fn show(__context__: &__Context__, value: T) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0:?}", value))
        })
    }
}
impl<__Context__, T: core::fmt::Debug> IsProviderFor<ShowerComponent, __Context__, (T)>
for ShowDebug {}
pub struct ShowDebug;
pub struct App;
impl DelegateComponent<ShowerComponent> for App {
    type Delegate = ShowDebug;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ShowerComponent, __Context__, __Params__> for App
where
    ShowDebug: IsProviderFor<ShowerComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ShowerComponent, u32> for App {}
impl __CheckApp<ShowerComponent, u32> for App {}
fn main() {}
