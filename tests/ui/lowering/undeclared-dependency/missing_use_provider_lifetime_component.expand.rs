#![feature(prelude_import)]
//! A higher-order provider of a component with a lifetime parameter, `CanGetName<'a>`, calls its
//! inner provider without `#[use_provider]`.
//!
//! The tool reports `[CGP-E031]` with no help: the bound needs the lifetime (`NameGetter<'a>`), and
//! the help, which writes the component's type parameters from the call, cannot write it, so a bare
//! `#[use_provider(Inner: NameGetter)]`, which fails with `E0106`, is not offered.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGetName<'a> {
    fn name(&self, prefix: &'a str) -> &'a str;
}
impl<'a, __Context__> CanGetName<'a> for __Context__
where
    __Context__: NameGetter<'a, __Context__>,
{
    fn name(&self, prefix: &'a str) -> &'a str {
        __Context__::name(self, prefix)
    }
}
pub trait NameGetter<
    'a,
    __Context__,
>: IsProviderFor<NameGetterComponent, __Context__, (Life<'a>)> {
    fn name(__context__: &__Context__, prefix: &'a str) -> &'a str;
}
impl<'a, __Provider__, __Context__> NameGetter<'a, __Context__> for __Provider__
where
    __Provider__: DelegateComponent<NameGetterComponent>
        + IsProviderFor<NameGetterComponent, __Context__, (Life<'a>)>,
    <__Provider__ as DelegateComponent<
        NameGetterComponent,
    >>::Delegate: NameGetter<'a, __Context__>,
{
    fn name(__context__: &__Context__, prefix: &'a str) -> &'a str {
        <__Provider__ as DelegateComponent<
            NameGetterComponent,
        >>::Delegate::name(__context__, prefix)
    }
}
pub struct NameGetterComponent;
impl<'a, __Context__> NameGetter<'a, __Context__> for UseContext
where
    __Context__: CanGetName<'a>,
{
    fn name(__context__: &__Context__, prefix: &'a str) -> &'a str {
        __Context__::name(__context__, prefix)
    }
}
impl<'a, __Context__> IsProviderFor<NameGetterComponent, __Context__, (Life<'a>)>
for UseContext
where
    __Context__: CanGetName<'a>,
{}
impl<'a, __Context__, __Components__, __Path__> NameGetter<'a, __Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: NameGetter<'a, __Context__>,
{
    fn name(__context__: &__Context__, prefix: &'a str) -> &'a str {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::name(__context__, prefix)
    }
}
impl<
    'a,
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<NameGetterComponent, __Context__, (Life<'a>)>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: NameGetter<'a, __Context__>,
{}
impl<'a, __Context__, Inner> NameGetter<'a, __Context__> for TrimmedName<Inner> {
    fn name(__context__: &__Context__, prefix: &'a str) -> &'a str {
        Inner::name(__context__, prefix).trim()
    }
}
impl<'a, __Context__, Inner> IsProviderFor<NameGetterComponent, __Context__, (Life<'a>)>
for TrimmedName<Inner> {}
pub struct TrimmedName<Inner>(pub ::core::marker::PhantomData<Inner>);
fn main() {}
