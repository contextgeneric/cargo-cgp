//! Calling `downcast` on the remainder of an earlier downcast.
//!
//! The remainder is an extractor, which takes `downcast_fields`; `downcast` needs the enum itself,
//! through `HasExtractor`: ``[CGP-E050] an extractor of `FooBarBaz` does not implement
//! `HasExtractor` ``. The `help` names `downcast_fields` with the import its trait needs.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E126

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
