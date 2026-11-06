#![feature(prelude_import)]
//! A namespace that inherits itself through a `crate::` path, `new SelfNs: crate::SelfNs`, which
//! `cgp_namespace!` does not check.
//!
//! The macro compares the parent with the namespace only for a bare name or a `self::` path, since
//! a longer path names an item it cannot resolve. The expansion reaches the compiler, which rejects
//! it with `E0207` on the generated `__Value__`. The tool must print that error as plain
//! `cargo check` prints it.
//!
//! CGP error class: none; a macro limit, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct __SelfNsComponents;
pub trait SelfNs<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> SelfNs<__Table__> for __Key__
where
    __Key__: crate::SelfNs<__SelfNsComponents>,
    __Key__: crate::SelfNs<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
fn main() {}
