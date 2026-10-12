//! `delegate_and_check_components!` with a generic key, `<I> FooKeyComponent<I>: AnyFoo`.
//!
//! Without `#[check_params(I)]` the derived check tests the key at unit parameters, which `AnyFoo`
//! does not provide, so rustc reports `AnyFoo: IsProviderFor<FooKeyComponent<I>, App>` and
//! suggests a `where` clause spliced into the entry. The tool reports `[CGP-E060]`, the check
//! naming no parameters, with `#[check_params(I)]` in the help; the generated `__CanUseApp` check
//! impl is not read as a wrapper trait the programmer wrote.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E060

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
