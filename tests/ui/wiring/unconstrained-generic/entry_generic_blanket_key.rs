//! A blanket entry, keyed by its own parameter `T`, whose provider names a second parameter the key
//! does not use: `<T, U> T: Wrap<U>`. `U` is unconstrained in the entry's impl and its
//! `IsProviderFor` twin (E0207).
//!
//! The tool leaves the two errors as rustc wrote them, with suggestions that do not compile: a key
//! that is a bare parameter has no rendering `[CGP-E019]` can name. The shape is an open issue of the
//! class.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/unconstrained-generic.md.

//@quality: usability
//@codes: none

use core::marker::PhantomData;

use cgp::prelude::*;

pub struct Wrap<U>(pub PhantomData<U>);

pub struct Person;

delegate_components! {
    Person {
        <T, U> T: Wrap<U>,
    }
}

fn main() {}
