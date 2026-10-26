#![feature(prelude_import)]
//! Two namespaces that each inherit another instantiation of themselves: `Ns<T>: Ns<u32>` and
//! `Ns<T>: Ns<Vec<T>>`.
//!
//! A namespace's every instantiation has the same parent rule, so a lookup that reaches the parent
//! asks the same namespace again, at `u32` forever or at a type that grows, and neither namespace
//! can resolve a key its table lacks. Stable `rustc` rejects both with an overflow, one of them
//! without a location; the pinned nightly, which the tool and the baseline run, accepts them, as it
//! accepts a namespace inheritance cycle. The tool must print what plain `cargo check` prints.
//!
//! CGP error class: none; a cycle visible in one invocation, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct __FixedNsComponents;
pub trait FixedNs<T, __Table__> {
    type Delegate;
}
impl<T, __Table__, __Key__, __Value__> FixedNs<T, __Table__> for __Key__
where
    __Key__: FixedNs<u32, __FixedNsComponents>,
    __Key__: FixedNs<u32, __Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
pub struct __GrowingNsComponents;
pub trait GrowingNs<T, __Table__> {
    type Delegate;
}
impl<T, __Table__, __Key__, __Value__> GrowingNs<T, __Table__> for __Key__
where
    __Key__: GrowingNs<Vec<T>, __GrowingNsComponents>,
    __Key__: GrowingNs<Vec<T>, __Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
fn main() {}
