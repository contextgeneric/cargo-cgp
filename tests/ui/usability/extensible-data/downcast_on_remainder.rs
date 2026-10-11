//! Calling `downcast` on the remainder of an earlier downcast.
//!
//! The remainder is an extractor, which takes `downcast_fields`. The output is an `E0599` naming the
//! unmet `HasExtractor` bound on `__PartialFooBarBaz<IsVoid, IsVoid, IsPresent>` and never points at
//! `downcast_fields`.
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
pub enum FooBarBaz {
    Foo(u64),
    Bar(String),
    Baz(bool),
}

#[derive(Debug, PartialEq, CgpData)]
pub enum Baz {
    Baz(bool),
}

fn main() {
    if let Err(remainder) = FooBarBaz::Baz(true).downcast(PhantomData::<FooBar>) {
        let _ = remainder.downcast(PhantomData::<Baz>);
    }
}
