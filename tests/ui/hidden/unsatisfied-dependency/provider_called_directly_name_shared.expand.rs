#![feature(prelude_import)]
//! A provider's own method called through its path, where a provider trait of another module
//! declares a function of the same name.
//!
//! `GreetHello::run(&Person {})` calls the `Greeter` provider trait's `run`, and `Person` lacks the
//! `name` field `GreetHello` reads. The module `other` defines a `Runner` provider trait whose
//! function is also named `run`, but this module does not import it. The provider-call anchor takes
//! only a provider trait in scope at the call, as the compiler's own lookup does, so the block names
//! the missing field rather than `Runner`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod other {
    use cgp::prelude::*;
    pub trait CanRun {
        fn run(&self) -> u8;
    }
    impl<__Context__> CanRun for __Context__
    where
        __Context__: Runner<__Context__>,
    {
        fn run(&self) -> u8 {
            __Context__::run(self)
        }
    }
    pub trait Runner<__Context__>: IsProviderFor<RunnerComponent, __Context__, ()> {
        fn run(__context__: &__Context__) -> u8;
    }
    impl<__Provider__, __Context__> Runner<__Context__> for __Provider__
    where
        __Provider__: DelegateComponent<RunnerComponent>
            + IsProviderFor<RunnerComponent, __Context__, ()>,
        <__Provider__ as DelegateComponent<
            RunnerComponent,
        >>::Delegate: Runner<__Context__>,
    {
        fn run(__context__: &__Context__) -> u8 {
            <__Provider__ as DelegateComponent<
                RunnerComponent,
            >>::Delegate::run(__context__)
        }
    }
    pub struct RunnerComponent;
    impl<__Context__> Runner<__Context__> for UseContext
    where
        __Context__: CanRun,
    {
        fn run(__context__: &__Context__) -> u8 {
            __Context__::run(__context__)
        }
    }
    impl<__Context__> IsProviderFor<RunnerComponent, __Context__, ()> for UseContext
    where
        __Context__: CanRun,
    {}
    impl<__Context__, __Components__, __Path__> Runner<__Context__>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<__Path__>>::Delegate: Runner<__Context__>,
    {
        fn run(__context__: &__Context__) -> u8 {
            <__Components__ as DelegateComponent<__Path__>>::Delegate::run(__context__)
        }
    }
    impl<
        __Context__,
        __Components__,
        __Path__,
    > IsProviderFor<RunnerComponent, __Context__, ()>
    for RedirectLookup<__Components__, __Path__>
    where
        __Components__: DelegateComponent<__Path__>,
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate: IsProviderFor<RunnerComponent, __Context__, ()>
            + Runner<__Context__>,
    {}
}
use cgp::prelude::*;
pub trait CanGreet {
    fn run(&self) -> String;
}
impl<__Context__> CanGreet for __Context__
where
    __Context__: Greeter<__Context__>,
{
    fn run(&self) -> String {
        __Context__::run(self)
    }
}
pub trait Greeter<__Context__>: IsProviderFor<GreeterComponent, __Context__, ()> {
    fn run(__context__: &__Context__) -> String;
}
impl<__Provider__, __Context__> Greeter<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<GreeterComponent>
        + IsProviderFor<GreeterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        GreeterComponent,
    >>::Delegate: Greeter<__Context__>,
{
    fn run(__context__: &__Context__) -> String {
        <__Provider__ as DelegateComponent<GreeterComponent>>::Delegate::run(__context__)
    }
}
pub struct GreeterComponent;
impl<__Context__> Greeter<__Context__> for UseContext
where
    __Context__: CanGreet,
{
    fn run(__context__: &__Context__) -> String {
        __Context__::run(__context__)
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
    fn run(__context__: &__Context__) -> String {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::run(__context__)
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
pub trait HasName {
    fn name(&self) -> &str;
}
impl<__Context__> HasName for __Context__
where
    __Context__: HasField<Symbol!("name"), Value = String>,
{
    fn name(&self) -> &str {
        self.get_field(::core::marker::PhantomData::<Symbol!("name")>).as_str()
    }
}
impl<__Context__> Greeter<__Context__> for GreetHello
where
    __Context__: HasName,
{
    fn run(__context__: &__Context__) -> String {
        __context__.name().to_owned()
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetHello
where
    __Context__: HasName,
{}
pub struct GreetHello;
pub struct Person {}
fn main() {
    let _ = GreetHello::run(&Person {});
}
