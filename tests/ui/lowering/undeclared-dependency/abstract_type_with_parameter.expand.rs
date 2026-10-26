#![feature(prelude_import)]
//! A provider body names `Self::Fault`, where the only abstract-type trait declaring `Fault` takes a
//! parameter of its own, `HasErrorTypeFor<Tag>`.
//!
//! The tool leaves rustc's `E0220` as written: `#[use_type(HasErrorTypeFor.Fault)]` would leave out
//! the trait's parameter, so the import is not one it can name.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasErrorTypeFor<Tag> {
    type Fault;
}
impl<__Context__, Tag> HasErrorTypeFor<Tag> for __Context__
where
    __Context__: FaultTypeProvider<__Context__, Tag>,
{
    type Fault = <__Context__ as FaultTypeProvider<__Context__, Tag>>::Fault;
}
pub trait FaultTypeProvider<
    __Context__,
    Tag,
>: IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)> {
    type Fault;
}
impl<__Provider__, __Context__, Tag> FaultTypeProvider<__Context__, Tag> for __Provider__
where
    __Provider__: DelegateComponent<FaultTypeProviderComponent>
        + IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)>,
    <__Provider__ as DelegateComponent<
        FaultTypeProviderComponent,
    >>::Delegate: FaultTypeProvider<__Context__, Tag>,
{
    type Fault = <<__Provider__ as DelegateComponent<
        FaultTypeProviderComponent,
    >>::Delegate as FaultTypeProvider<__Context__, Tag>>::Fault;
}
pub struct FaultTypeProviderComponent;
impl<__Context__, Tag> FaultTypeProvider<__Context__, Tag> for UseContext
where
    __Context__: HasErrorTypeFor<Tag>,
{
    type Fault = <__Context__ as HasErrorTypeFor<Tag>>::Fault;
}
impl<__Context__, Tag> IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)>
for UseContext
where
    __Context__: HasErrorTypeFor<Tag>,
{}
impl<__Context__, Tag, __Components__, __Path__> FaultTypeProvider<__Context__, Tag>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Tag)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Tag)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Tag)>>::Output,
    >>::Delegate: FaultTypeProvider<__Context__, Tag>,
{
    type Fault = <<__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Tag)>>::Output,
    >>::Delegate as FaultTypeProvider<__Context__, Tag>>::Fault;
}
impl<
    __Context__,
    Tag,
    __Components__,
    __Path__,
> IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Tag)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Tag)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Tag)>>::Output,
    >>::Delegate: IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)>
        + FaultTypeProvider<__Context__, Tag>,
{}
impl<Fault, __Context__, Tag> FaultTypeProvider<__Context__, Tag> for UseType<Fault> {
    type Fault = Fault;
}
impl<
    Fault,
    __Context__,
    Tag,
> IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)> for UseType<Fault> {}
impl<__Provider__, Fault, __Context__, Tag> FaultTypeProvider<__Context__, Tag>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, FaultTypeProviderComponent, Type = Fault>,
{
    type Fault = Fault;
}
impl<
    __Provider__,
    Fault,
    __Context__,
    Tag,
> IsProviderFor<FaultTypeProviderComponent, __Context__, (Tag)>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, FaultTypeProviderComponent, Type = Fault>,
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
impl<__Context__> Greeter<__Context__> for GreetHello {
    fn greet(__context__: &__Context__) -> String {
        let _fault: Option<__Context__::Fault> = None;
        String::new()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello {}
pub struct GreetHello;
fn main() {}
