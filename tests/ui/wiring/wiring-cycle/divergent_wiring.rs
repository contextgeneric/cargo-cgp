//! A wiring whose lookup grows without repeating: `ShowNested` implements `Shower<T>` by requiring
//! `CanShow<Vec<T>>` of the context, so checking `CanShow<u32>` asks for `CanShow<Vec<u32>>`, then
//! `CanShow<Vec<Vec<u32>>>`, and so on. The trait solver overflows (`E0275`). No obligation
//! repeats, so the resolver's cycle guard never fires; its walk stops at the depth cap, which flags
//! the walk as cut, and the flag lets the tool word the overflow as a wiring that never resolves
//! (`[CGP-E020]`).
//!
//! The output is `usability`: the second help blames a delegation back to the context, as with
//! `UseContext`, which is not this mistake.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the depth cap is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.

//@quality: usability
//@codes: CGP-E020

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

check_components! {
    App {
        ShowerComponent: u32,
    }
}

fn main() {}
