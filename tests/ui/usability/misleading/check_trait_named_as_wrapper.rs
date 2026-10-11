//! `delegate_and_check_components!` with a generic key, `<I> FooKeyComponent<I>: AnyFoo`.
//!
//! The failure is reshaped as `[CGP-E009] the trait `__CanUseApp<FooKeyComponent<I>, ()>``, presenting
//! the generated check trait as a trait the user wrote, over a `[CGP-E107]` leaf.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component {
    name: FooKeyComponent<I>,
    provider: FooProvider,
}]
pub trait CanFoo<I> {
    fn foo(&self, tag: PhantomData<I>) -> u8;
}

#[cgp_impl(new AnyFoo: FooKeyComponent<I>)]
impl<I> FooProvider<I> {
    fn foo(&self, _tag: PhantomData<I>) -> u8 {
        0
    }
}

pub struct App;

// error[E0277]: the derived check tests `FooKeyComponent<I>` at unit parameters
delegate_and_check_components! {
    App {
        <I> FooKeyComponent<I>: AnyFoo,
    }
}

fn main() {}
