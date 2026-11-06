#![feature(prelude_import)]
//! `#[uses(Reader<'static, Text>)]` on a `#[cgp_impl]` provider, naming the provider trait of a
//! component with a lifetime where its consumer trait belongs.
//!
//! The provider trait `Reader<'a, Context, Source>` takes the context after the lifetime, so the
//! type argument written fills the context's place and rustc reports `E0107`. The tool reshapes it
//! into `[CGP-E037]` as it does for a component without parameters, counting only the type
//! arguments, and its help names `#[uses(CanRead<'static, Text>)]`.
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
pub trait CanGreet {
    fn greet(&self) -> String;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn greet(&self) -> String {
        __Context__::greet(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn greet(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn greet(__context__: &__Context__) -> String {
        __Context__::greet(__context__)
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for UseContext
where
    __Context__: CanGreet,
{}
impl<__Context__, __Components__, __Path__> Greeter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Greeter<__Context__>,
{
    fn greet(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::greet(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, ()>
        + Greeter<__Context__>,
{}
pub struct Text;
impl<__Context__> Greeter<__Context__> for GreetByReading
where
    __Context__: Reader<'static, Text>,
{
    fn greet(__context__: &__Context__) -> String {
        __context__.read(&Text).to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetByReading
where
    __Context__: Reader<'static, Text>,
{}
pub struct GreetByReading;
fn main() {}
