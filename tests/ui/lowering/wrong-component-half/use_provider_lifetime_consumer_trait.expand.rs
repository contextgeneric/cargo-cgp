#![feature(prelude_import)]
//! A `#[use_provider]` bound naming the consumer trait of a component with a lifetime parameter.
//!
//! `#[use_provider(Inner: CanRead<'a, Source>)]` should name `Reader<'a, Source>`. The attribute
//! inserts the context after the lifetime, generating `Inner: CanRead<'a, __Context__, Source>`,
//! so the consumer trait receives one type argument too many and rustc reports `E0107`. The bound
//! sits on the inner provider with the context as its first type argument, the shape the attribute
//! writes, so the tool reshapes it into `[CGP-E034]` as it does for a component without lifetimes.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanRead<'a, Source> {
    fn read(&self, source: &'a Source) -> &'a str;
}
impl<'a, __Context__, Source> CanRead<'a, Source> for __Context__
where
    __Context__: Reader<'a, __Context__, Source>,
{
    fn read(&self, source: &'a Source) -> &'a str {
        __Context__::read(self, source)
    }
}
pub trait Reader<
    'a,
    __Context__,
    Source,
>: IsProviderFor<ReaderComponent, __Context__, (Life<'a>, Source)> {
    fn read(__context__: &__Context__, source: &'a Source) -> &'a str;
}
impl<'a, __Provider__, __Context__, Source> Reader<'a, __Context__, Source>
for __Provider__
where
    __Provider__: DelegateComponent<ReaderComponent>
        + IsProviderFor<ReaderComponent, __Context__, (Life<'a>, Source)>,
    <__Provider__ as DelegateComponent<
        ReaderComponent,
    >>::Delegate: Reader<'a, __Context__, Source>,
{
    fn read(__context__: &__Context__, source: &'a Source) -> &'a str {
        <__Provider__ as DelegateComponent<
            ReaderComponent,
        >>::Delegate::read(__context__, source)
    }
}
pub struct ReaderComponent;
impl<'a, __Context__, Source> Reader<'a, __Context__, Source> for UseContext
where
    __Context__: CanRead<'a, Source>,
{
    fn read(__context__: &__Context__, source: &'a Source) -> &'a str {
        __Context__::read(__context__, source)
    }
}
impl<
    'a,
    __Context__,
    Source,
> IsProviderFor<ReaderComponent, __Context__, (Life<'a>, Source)> for UseContext
where
    __Context__: CanRead<'a, Source>,
{}
impl<'a, __Context__, Source, __Components__, __Path__> Reader<'a, __Context__, Source>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Source)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Source)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Source)>>::Output,
    >>::Delegate: Reader<'a, __Context__, Source>,
{
    fn read(__context__: &__Context__, source: &'a Source) -> &'a str {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Source)>>::Output,
        >>::Delegate::read(__context__, source)
    }
}
impl<
    'a,
    __Context__,
    Source,
    __Components__,
    __Path__,
> IsProviderFor<ReaderComponent, __Context__, (Life<'a>, Source)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Source)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Source)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Source)>>::Output,
    >>::Delegate: Reader<'a, __Context__, Source>,
{}
impl<'a, __Context__, Inner, Source> Reader<'a, __Context__, Source>
for ReadThrough<Inner>
where
    Inner: CanRead<'a, __Context__, Source>,
{
    fn read(__context__: &__Context__, source: &'a Source) -> &'a str {
        Inner::read(__context__, source)
    }
}
impl<
    'a,
    __Context__,
    Inner,
    Source,
> IsProviderFor<ReaderComponent, __Context__, (Life<'a>, Source)> for ReadThrough<Inner>
where
    Inner: CanRead<'a, __Context__, Source>,
{}
pub struct ReadThrough<Inner>(pub ::core::marker::PhantomData<Inner>);
fn main() {}
