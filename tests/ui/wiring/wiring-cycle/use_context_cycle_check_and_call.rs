//! A component wired to `UseContext` with no other implementation, reached by a check and a call.
//!
//! The compiler reports the loop twice: an `E0275` on `Person: CanUseComponent<GreeterComponent>`
//! at the check entry, and one on `Person: CanGreet` at the call. Both walk to the same cycle, so
//! both read as the same `[CGP-E020]`, and the emitter's de-duplication keeps the first and drops
//! the call's as a repeat of one mistake, as it does for a missing field reported at several sites.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).

//@quality: acceptable
//@codes: CGP-E020

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: UseContext,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {
    Person.greet();
}
