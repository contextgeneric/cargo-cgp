//! Downcasting into a target with a variant the source lacks.
//!
//! `FooBar` downcast into `FooQux { Foo, Qux }` fails on the private `FieldsExtractor` recursion over
//! `Enum! { Qux(char) }`. The output never says `FooBar` has no `Qux` variant, and the reference
//! documents quote an `ExtractField` bound on the partial enum instead.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
