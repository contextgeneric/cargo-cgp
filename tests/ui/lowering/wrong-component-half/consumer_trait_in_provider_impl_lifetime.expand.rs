#![feature(prelude_import)]
//! A `#[cgp_impl]` header naming the consumer trait of a component with a lifetime parameter.
//!
//! The same mistake as `consumer_trait_in_provider_impl_generic`, on `CanRead<'a, Source>`. The
//! macro writes the header as `CanRead<'a, __Context__, Text>`, placing the context after the
//! lifetime, so rustc reports `E0107`. The tool reshapes it into `[CGP-E032]`, and the help writes
//! the provider trait with the arguments the header gave the consumer trait, `Reader<'a, Text>`,
//! without the inserted context.
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
///Component marker for [`CanRead`].
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
pub struct Text;
impl<'a, __Context__> CanRead<'a, __Context__, Text> for ReadText {
    fn read(__context__: &__Context__, _source: &'a Text) -> &'a str {
        ""
    }
}
impl<'a, __Context__> IsProviderFor<CanReadComponent, __Context__, (Life<'a>, Text)>
for ReadText {}
///Provider `ReadText` for [`CanRead`].
pub struct ReadText;
fn main() {}
