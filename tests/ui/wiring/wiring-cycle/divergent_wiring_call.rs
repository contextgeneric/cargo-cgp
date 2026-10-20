//! A wiring whose lookup grows without repeating, reached by a method call rather than a check.
//!
//! `ShowNested` implements `Shower<T>` by requiring `CanShow<Vec<T>>` of the context, so the call
//! `App.show(1u32)` asks for `CanShow<Vec<u32>>`, then `CanShow<Vec<Vec<u32>>>`, and so on, and the
//! trait solver overflows (`E0275`) on the consumer trait at the call. The tool walks the call's
//! consumer obligation as it walks a check entry's, and words the overflow as a lookup that grows
//! (`[CGP-E021]`), with a help naming `ShowNested`.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`).

//@quality: acceptable
//@codes: CGP-E021

use cgp::prelude::*;

#[cgp_component(Shower)]
pub trait CanShow<T> {
    fn show(&self, value: T);
}

#[cgp_impl(new ShowNested)]
#[uses(CanShow<Vec<T>>)]
impl<T> Shower<T> {
    fn show(&self, value: T) {
        self.show(vec![value]);
    }
}

pub struct App;

delegate_components! {
    App {
        ShowerComponent: ShowNested,
    }
}

fn main() {
    App.show(1u32);
}
