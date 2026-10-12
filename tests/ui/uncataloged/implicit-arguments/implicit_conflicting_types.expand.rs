#![feature(prelude_import)]
//! Two methods of one provider reading the same implicit field as two different types.
//!
//! An `E0284` and an `E0282`, both on the provider name; nothing says the methods disagree on the
//! field's type. rustc reports the `E0284` twice, and the tool merges the repeat, which differed
//! only in a note the tool renamed, into the first block.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCount {
    fn small(&self) -> u32;
    fn large(&self) -> u64;
}
impl<__Context__> CanCount for __Context__
where
    __Context__: Counter<__Context__>,
{
    fn small(&self) -> u32 {
        __Context__::small(self)
    }
    fn large(&self) -> u64 {
        __Context__::large(self)
    }
}
pub trait Counter<__Context__>: IsProviderFor<CounterComponent, __Context__, ()> {
    fn small(__context__: &__Context__) -> u32;
    fn large(__context__: &__Context__) -> u64;
}
impl<__Provider__, __Context__> Counter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<CounterComponent>
        + IsProviderFor<CounterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        CounterComponent,
    >>::Delegate: Counter<__Context__>,
{
    fn small(__context__: &__Context__) -> u32 {
        <__Provider__ as DelegateComponent<
            CounterComponent,
        >>::Delegate::small(__context__)
    }
    fn large(__context__: &__Context__) -> u64 {
        <__Provider__ as DelegateComponent<
            CounterComponent,
        >>::Delegate::large(__context__)
    }
}
pub struct CounterComponent;
impl<__Context__> Counter<__Context__> for UseContext
where
    __Context__: CanCount,
{
    fn small(__context__: &__Context__) -> u32 {
        __Context__::small(__context__)
    }
    fn large(__context__: &__Context__) -> u64 {
        __Context__::large(__context__)
    }
}
impl<__Context__> IsProviderFor<CounterComponent, __Context__, ()> for UseContext
where
    __Context__: CanCount,
{}
impl<__Context__, __Components__, __Path__> Counter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Counter<__Context__>,
{
    fn small(__context__: &__Context__) -> u32 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::small(__context__)
    }
    fn large(__context__: &__Context__) -> u64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::large(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<CounterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<CounterComponent, __Context__, ()>
        + Counter<__Context__>,
{}
impl<__Context__> Counter<__Context__> for ReadCount
where
    __Context__: HasField<Symbol!("count"), Value = u32>
        + HasField<Symbol!("count"), Value = u64>,
{
    fn small(__context__: &__Context__) -> u32 {
        let count: u32 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("count")>)
            .clone();
        count
    }
    fn large(__context__: &__Context__) -> u64 {
        let count: u64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("count")>)
            .clone();
        count
    }
}
impl<__Context__> IsProviderFor<CounterComponent, __Context__, ()> for ReadCount
where
    __Context__: HasField<Symbol!("count"), Value = u32>
        + HasField<Symbol!("count"), Value = u64>,
{}
pub struct ReadCount;
fn main() {}
