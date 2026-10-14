//! Downcasting into a target with a variant the source lacks.
//!
//! `FooBar` downcast into `FooQux { Foo, Qux }` needs every variant of `FooQux` in `FooBar`. The cast's
//! extractor recursion fails on `Qux`, which `FooBar` lacks, and the tool names it:
//! ``[CGP-E050] missing variant `Qux` on `FooBar` ``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E120

use cgp::core::field::impls::CanDowncast;
use cgp::prelude::*;

#[derive(Debug, PartialEq, CgpData)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

#[derive(Debug, PartialEq, CgpData)]
pub enum FooQux {
    Foo(u64),
    Qux(char),
}

fn main() {
    let _ = FooBar::Foo(1).downcast(PhantomData::<FooQux>);
}
