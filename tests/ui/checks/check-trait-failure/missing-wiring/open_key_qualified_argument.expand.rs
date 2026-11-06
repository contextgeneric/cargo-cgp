#![feature(prelude_import)]
//! An `open` dispatch key whose value type takes a type two modules share a name for.
//!
//! rustc prints `a::Item` qualified, because `b::Item` shares its name. The missing key is
//! `@GreeterComponent.Wrap<a::Item>`: the segment's own head is a plain type, and only its argument
//! holds a path.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub mod a {
    pub struct Item;
}
pub mod b {
    pub struct Item;
}
pub struct Wrap<T>(pub T);
pub trait CanGreet<Input> {
    fn greet(&self, input: Input);
}
impl<__Context__, Input> CanGreet<Input> for __Context__
where
    __Context__: Greeter<__Context__, Input>,
{
    fn greet(&self, input: Input) {
        __Context__::greet(self, input)
    }
}
pub trait Greeter<
    __Context__,
    Input,
>: IsProviderFor<GreeterComponent, __Context__, (Input)> {
    fn greet(__context__: &__Context__, input: Input);
}
impl<__Provider__, __Context__, Input> Greeter<__Context__, Input> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, (Input)>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__, Input>,
{
    fn greet(__context__: &__Context__, input: Input) {
        <__Provider__ as DelegateComponent<
            GreeterComponent,
        >>::Delegate::greet(__context__, input)
    }
}
pub struct GreeterComponent;
impl<__Context__, Input> Greeter<__Context__, Input> for UseContext
where
    __Context__: CanGreet<Input>,
{
    fn greet(__context__: &__Context__, input: Input) {
        __Context__::greet(__context__, input)
    }
}
impl<__Context__, Input> IsProviderFor<GreeterComponent, __Context__, (Input)>
for UseContext
where
    __Context__: CanGreet<Input>,
{}
impl<__Context__, Input, __Components__, __Path__> Greeter<__Context__, Input>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Input)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Input)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Input)>>::Output,
    >>::Delegate: Greeter<__Context__, Input>,
{
    fn greet(__context__: &__Context__, input: Input) {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Input)>>::Output,
        >>::Delegate::greet(__context__, input)
    }
}
impl<
    __Context__,
    Input,
    __Components__,
    __Path__,
> IsProviderFor<GreeterComponent, __Context__, (Input)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Input)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Input)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Input)>>::Output,
    >>::Delegate: IsProviderFor<GreeterComponent, __Context__, (Input)>
        + Greeter<__Context__, Input>,
{}
impl<__Context__> Greeter<__Context__, Wrap<b::Item>> for GreetB {
    fn greet(__context__: &__Context__, _input: Wrap<b::Item>) {}
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, (Wrap<b::Item>)>
for GreetB {}
pub struct GreetB;
pub struct App;
impl DelegateComponent<GreeterComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@GreeterComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<GreeterComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@GreeterComponent),
    >: IsProviderFor<GreeterComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<GreeterComponent, PathCons<Wrap<b::Item>, __Wildcard__>>>
for App {
    type Delegate = GreetB;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<GreeterComponent, PathCons<Wrap<b::Item>, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    GreetB: IsProviderFor<
        PathCons<GreeterComponent, PathCons<Wrap<b::Item>, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<GreeterComponent, Wrap<a::Item>> for App {}
fn main() {}
