#![feature(prelude_import)]
//! A namespace that inherits itself, `new SelfNs: SelfNs`.
//!
//! `E0207` on `__Value__` across the whole block; decidable at macro time.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
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
    __Key__: SelfNs<__SelfNsComponents>,
    __Key__: SelfNs<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
fn main() {}
