//! `delegate_and_check_components!` with a generic key, `<I> FooKeyComponent<I>: AnyFoo`.
//!
//! Without `#[check_params(I)]` the derived check tests the key at unit parameters, which `AnyFoo`
//! does not provide, so rustc reports `AnyFoo: IsProviderFor<FooKeyComponent<I>, App>`. The tool
//! leaves it as rustc wrote it: the generated `__CanUseApp` check impl is not read as a wrapper
//! trait the programmer wrote. Nothing names `#[check_params(I)]` as the fix.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
