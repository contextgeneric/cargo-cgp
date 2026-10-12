#![feature(prelude_import)]
//! A provider for a component with `#[extend(TagFor<u32>)]` that omits `#[uses(TagFor<u32>)]`.
//!
//! The component requires the generic trait `TagFor<u32>` of every context, so rustc reports
//! `E0277` on `__Context__: TagFor<u32>` at the `#[cgp_impl]` attribute, with a help about the
//! `HasField` bound inside `TagFor`. The tool reshapes it into `[CGP-E036]`, writing the requirement
//! with its argument in the `#[uses(TagFor<u32>)]` help.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait TagFor<T> {
    fn tag_for(&self) -> u64;
}
impl<__Context__, T> TagFor<T> for __Context__
where
    Self: HasField<Symbol!("count"), Value = u64>,
{
    fn tag_for(&self) -> u64 {
        let count: &u64 = self
            .get_field(::core::marker::PhantomData::<Symbol!("count")>);
        let _ = core::marker::PhantomData::<T>;
        *count
    }
}
pub trait CanTag: TagFor<u32> {
    fn tag(&self) -> u64;
}
impl<__Context__> CanTag for __Context__
where
    __Context__: TagFor<u32>,
    __Context__: Tagger<__Context__>,
{
    fn tag(&self) -> u64 {
        __Context__::tag(self)
    }
}
pub trait Tagger<__Context__>: IsProviderFor<TaggerComponent, __Context__, ()>
where
    __Context__: TagFor<u32>,
{
    fn tag(__context__: &__Context__) -> u64;
}
impl<__Provider__, __Context__> Tagger<__Context__> for __Provider__
where
    __Context__: TagFor<u32>,
    __Provider__: DelegateComponent<TaggerComponent>
        + IsProviderFor<TaggerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<TaggerComponent>>::Delegate: Tagger<__Context__>,
{
    fn tag(__context__: &__Context__) -> u64 {
        <__Provider__ as DelegateComponent<TaggerComponent>>::Delegate::tag(__context__)
    }
}
pub struct TaggerComponent;
impl<__Context__> Tagger<__Context__> for UseContext
where
    __Context__: TagFor<u32>,
    __Context__: CanTag,
{
    fn tag(__context__: &__Context__) -> u64 {
        __Context__::tag(__context__)
    }
}
impl<__Context__> IsProviderFor<TaggerComponent, __Context__, ()> for UseContext
where
    __Context__: TagFor<u32>,
    __Context__: CanTag,
{}
impl<__Context__, __Components__, __Path__> Tagger<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: TagFor<u32>,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Tagger<__Context__>,
{
    fn tag(__context__: &__Context__) -> u64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::tag(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TaggerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: TagFor<u32>,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TaggerComponent, __Context__, ()> + Tagger<__Context__>,
{}
impl<__Context__> Tagger<__Context__> for TagIt {
    fn tag(__context__: &__Context__) -> u64 {
        0
    }
}
impl<__Context__> IsProviderFor<TaggerComponent, __Context__, ()> for TagIt {}
pub struct TagIt;
fn main() {}
