#![feature(prelude_import)]
//! A blanket entry, keyed by its own parameter `T`, whose provider names a second parameter the key
//! does not use: `<T, U> T: Wrap<U>`. `U` is unconstrained in the entry's impl and its
//! `IsProviderFor` twin (E0207).
//!
//! The tool leaves the two errors as rustc wrote them, with suggestions that do not compile: a key
//! that is a bare parameter has no rendering `[CGP-E019]` can name. The shape is an open issue of the
//! class.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
use cgp::prelude::*;
pub struct Wrap<U>(pub PhantomData<U>);
pub struct Person;
impl<T, U> DelegateComponent<T> for Person {
    type Delegate = Wrap<U>;
}
impl<T, U, __Context__, __Params__: ?Sized> IsProviderFor<T, __Context__, __Params__>
for Person
where
    Wrap<U>: IsProviderFor<T, __Context__, __Params__>,
{}
fn main() {}
