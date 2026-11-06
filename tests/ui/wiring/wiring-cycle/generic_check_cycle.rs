//! A check with a generic parameter on a wiring cycle.
//!
//! `App<T>` wires `GreeterComponent` to `UseContext`, so the lookup leads back to itself. The
//! resolver solves under an empty parameter environment, where the solver does not prove a wiring
//! lookup that names the check's own `T`, so it cannot read the provider the entry wires, and must
//! not name one. The overflow keeps rustc's wording.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct App<T>(pub PhantomData<T>);

delegate_components! {
    <T> App<T> {
        GreeterComponent: UseContext,
    }
}

check_components! {
    <T> App<T> {
        GreeterComponent,
    }
}

fn main() {}
