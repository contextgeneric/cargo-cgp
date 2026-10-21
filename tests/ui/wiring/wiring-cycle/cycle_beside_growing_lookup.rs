//! A wiring whose lookup both grows and loops: `ShowNested` provides `CanShow<T>` by requiring
//! `CanShow<Vec<T>>`, a larger type at every step, and `CanGreet`, which `App` wires to
//! `UseContext`, so that requirement leads back to itself.
//!
//! The walk follows the growing requirement first, to its depth cap, and meets the loop beside it at
//! every step. A cycle explains a lookup that never resolves more exactly than a depth cut, so the
//! overflow reads as `[CGP-E020]`, and the help names `UseContext` as the provider that closes the
//! loop.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`).

//@quality: acceptable
//@codes: CGP-E020

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_component(Shower)]
pub trait CanShow<T> {
    fn show(&self, value: T);
}

#[cgp_impl(new ShowNested)]
#[uses(CanShow<Vec<T>>, CanGreet)]
impl<T> Shower<T> {
    fn show(&self, value: T) {
        self.greet();
        self.show(vec![value]);
    }
}

pub struct App;

delegate_components! {
    App {
        ShowerComponent: ShowNested,
        GreeterComponent: UseContext,
    }
}

check_components! {
    App {
        ShowerComponent: u32,
    }
}

fn main() {}
