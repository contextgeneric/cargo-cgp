//! Upcasting into a target whose same-named variant carries another payload type.
//!
//! `FooBar::Foo(u64)` into `FooBarBaz::Foo(u32)` is an `E0271` on
//! `<FooBarBaz as FromVariant<Symbol!("Foo")>>::Value == u64` with a note on the target's `u32`.
//! Readable, but uncoded and worded around the internal `FieldsExtractor` recursion.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;

#[derive(Debug, PartialEq, CgpData)]
pub enum FooBar {
    Foo(u64),
    Bar(String),
}

#[derive(Debug, PartialEq, CgpData)]
pub enum FooBarBaz {
    Foo(u32),
    Bar(String),
    Baz(bool),
}

fn main() {
    let _ = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);
}
