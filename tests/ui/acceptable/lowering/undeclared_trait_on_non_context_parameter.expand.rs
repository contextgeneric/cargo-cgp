#![feature(prelude_import)]
//! A `#[cgp_impl]` provider body that calls a trait method on a *non-context* parameter, which the
//! `[CGP-E012]` reshaping must leave alone.
//!
//! `ReportValue` is generic over the component's `Value` parameter and calls `value.describe()`,
//! a `#[cgp_fn]` trait `Value` is not bounded by. The call fails with the same `E0599` shape as an
//! undeclared trait on the context (compare
//! [`undeclared_uses_trait_in_cgp_impl`](undeclared_uses_trait_in_cgp_impl.rs)), but the fix is a
//! bound on `Value`, which `#[uses(…)]` cannot express, since it bounds the context. So
//! `resolve::detect_undeclared_trait` requires the failing call's receiver to be the impl's generic
//! context, and this error passes through as rustc wrote it. A `[CGP-E012]` here would be a
//! regression that hands the reader a wrong fix.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
trait Describe {
    fn describe(&self) -> String;
}
impl<__Context__> Describe for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn describe(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
pub trait CanReport<Value> {
    fn report(&self, value: &Value) -> String;
}
impl<__Context__, Value> CanReport<Value> for __Context__
where
    __Context__: Reporter<__Context__, Value>,
{
    fn report(&self, value: &Value) -> String {
        __Context__::report(self, value)
    }
}
pub trait Reporter<
    __Context__,
    Value,
>: IsProviderFor<ReporterComponent, __Context__, (Value)> {
    fn report(__context__: &__Context__, value: &Value) -> String;
}
impl<__Provider__, __Context__, Value> Reporter<__Context__, Value> for __Provider__
where
    __Provider__: DelegateComponent<ReporterComponent>
        + IsProviderFor<ReporterComponent, __Context__, (Value)>,
    <__Provider__ as DelegateComponent<
        ReporterComponent,
    >>::Delegate: Reporter<__Context__, Value>,
{
    fn report(__context__: &__Context__, value: &Value) -> String {
        <__Provider__ as DelegateComponent<
            ReporterComponent,
        >>::Delegate::report(__context__, value)
    }
}
pub struct ReporterComponent;
impl<__Context__, Value> Reporter<__Context__, Value> for UseContext
where
    __Context__: CanReport<Value>,
{
    fn report(__context__: &__Context__, value: &Value) -> String {
        __Context__::report(__context__, value)
    }
}
impl<__Context__, Value> IsProviderFor<ReporterComponent, __Context__, (Value)>
for UseContext
where
    __Context__: CanReport<Value>,
{}
impl<__Context__, Value, __Components__, __Path__> Reporter<__Context__, Value>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: Reporter<__Context__, Value>,
{
    fn report(__context__: &__Context__, value: &Value) -> String {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Value)>>::Output,
        >>::Delegate::report(__context__, value)
    }
}
impl<
    __Context__,
    Value,
    __Components__,
    __Path__,
> IsProviderFor<ReporterComponent, __Context__, (Value)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Value)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Value)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Value)>>::Output,
    >>::Delegate: IsProviderFor<ReporterComponent, __Context__, (Value)>
        + Reporter<__Context__, Value>,
{}
impl<__Context__, Value> Reporter<__Context__, Value> for ReportValue {
    fn report(__context__: &__Context__, value: &Value) -> String {
        value.describe()
    }
}
impl<__Context__, Value> IsProviderFor<ReporterComponent, __Context__, (Value)>
for ReportValue {}
pub struct ReportValue;
fn main() {}
