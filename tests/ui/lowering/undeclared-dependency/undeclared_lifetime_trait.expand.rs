#![feature(prelude_import)]
//! A `#[cgp_fn]` body calls a method of a consumer trait with a lifetime parameter,
//! `CanGetPrefix<'a>`, without `#[uses]`.
//!
//! The tool reports `[CGP-E030]` with no help: the help writes only type arguments, and the bound
//! needs the lifetime.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanGetPrefix<'a> {
    fn prefix(&self, text: &'a str) -> &'a str;
}
impl<'a, __Context__> CanGetPrefix<'a> for __Context__
where
    __Context__: PrefixGetter<'a, __Context__>,
{
    fn prefix(&self, text: &'a str) -> &'a str {
        __Context__::prefix(self, text)
    }
}
pub trait PrefixGetter<
    'a,
    __Context__,
>: IsProviderFor<PrefixGetterComponent, __Context__, (Life<'a>)> {
    fn prefix(__context__: &__Context__, text: &'a str) -> &'a str;
}
impl<'a, __Provider__, __Context__> PrefixGetter<'a, __Context__> for __Provider__
where
    __Provider__: DelegateComponent<PrefixGetterComponent>
        + IsProviderFor<PrefixGetterComponent, __Context__, (Life<'a>)>,
    <__Provider__ as DelegateComponent<
        PrefixGetterComponent,
    >>::Delegate: PrefixGetter<'a, __Context__>,
{
    fn prefix(__context__: &__Context__, text: &'a str) -> &'a str {
        <__Provider__ as DelegateComponent<
            PrefixGetterComponent,
        >>::Delegate::prefix(__context__, text)
    }
}
///Component marker for [`CanGetPrefix`].
pub struct PrefixGetterComponent;
impl<'a, __Context__> PrefixGetter<'a, __Context__> for UseContext
where
    __Context__: CanGetPrefix<'a>,
{
    fn prefix(__context__: &__Context__, text: &'a str) -> &'a str {
        __Context__::prefix(__context__, text)
    }
}
impl<'a, __Context__> IsProviderFor<PrefixGetterComponent, __Context__, (Life<'a>)>
for UseContext
where
    __Context__: CanGetPrefix<'a>,
{}
impl<'a, __Context__, __Components__, __Path__> PrefixGetter<'a, __Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: PrefixGetter<'a, __Context__>,
{
    fn prefix(__context__: &__Context__, text: &'a str) -> &'a str {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::prefix(__context__, text)
    }
}
impl<
    'a,
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<PrefixGetterComponent, __Context__, (Life<'a>)>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: PrefixGetter<'a, __Context__>,
{}
///The trait for the `short_prefix` operation.
pub trait ShortPrefix<'a> {
    ///The `short_prefix` operation.
    fn short_prefix(&self, text: &'a str) -> &'a str;
}
impl<'a, __Context__> ShortPrefix<'a> for __Context__ {
    fn short_prefix(&self, text: &'a str) -> &'a str {
        self.prefix(text)
    }
}
fn main() {}
