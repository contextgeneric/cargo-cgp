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

use cgp::prelude::*;

#[cgp_fn]
fn describe(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_component(Reporter)]
pub trait CanReport<Value> {
    fn report(&self, value: &Value) -> String;
}

#[cgp_impl(new ReportValue)]
impl<Value> Reporter<Value> {
    fn report(&self, value: &Value) -> String {
        value.describe()
    }
}

fn main() {}
