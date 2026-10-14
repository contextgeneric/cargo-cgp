//! Upcasting into a target whose same-named variant carries another payload type.
//!
//! `FooBar::Foo(u64)` into `FooBarBaz::Foo(u32)` is an `E0271` on
//! `<FooBarBaz as FromVariant<Symbol!("Foo")>>::Value == u64`. The tool reads the payload types:
//! ``[CGP-E050] variant `Foo` on `FooBarBaz` has type `u32`, but `u64` is required``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E122

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
