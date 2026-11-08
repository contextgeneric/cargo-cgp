#![feature(prelude_import)]
//! Two namespaces that inherit from each other, which both compilers on the pinned nightly accept.
//!
//! Stable `rustc` rejects the cycle with an `E0275` overflow at each `cgp_namespace!` definition.
//! On the pinned nightly both the default solver and the next-generation solver the tool uses accept
//! it, so plain `cargo check` and `cargo cgp check` both compile it clean. That the tool reports
//! nothing here is a documented limitation: it checks only what the next solver rejects. The
//! fixture's exit-status check fails if a later nightly makes the two compilers disagree.
//!
//! CGP error class: namespace inheritance cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/namespace-inheritance-cycle.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
///Wiring table for the `NamespaceA` namespace.
pub struct __NamespaceAComponents;
///The `NamespaceA` namespace's wiring table.
pub trait NamespaceA<__Table__> {
    ///The provider table the `NamespaceA` namespace delegates to.
    type Delegate;
}
impl<__Table__, __Key__, __Value__> NamespaceA<__Table__> for __Key__
where
    __Key__: NamespaceB<__NamespaceAComponents>,
    __Key__: NamespaceB<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
///Wiring table for the `NamespaceB` namespace.
pub struct __NamespaceBComponents;
///The `NamespaceB` namespace's wiring table.
pub trait NamespaceB<__Table__> {
    ///The provider table the `NamespaceB` namespace delegates to.
    type Delegate;
}
impl<__Table__, __Key__, __Value__> NamespaceB<__Table__> for __Key__
where
    __Key__: NamespaceA<__NamespaceBComponents>,
    __Key__: NamespaceA<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
fn main() {}
