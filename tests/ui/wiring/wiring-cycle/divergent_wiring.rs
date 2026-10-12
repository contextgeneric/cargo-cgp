//! A wiring whose lookup grows without repeating: `ShowNested` implements `Shower<T>` by requiring
//! `CanShow<Vec<T>>` of the context, so checking `CanShow<u32>` asks for `CanShow<Vec<u32>>`, then
//! `CanShow<Vec<Vec<u32>>>`, and so on. The trait solver overflows (`E0275`). No obligation
//! repeats, so the resolver's cycle guard never fires; its walk stops at the depth cap, and the tool
//! words the overflow as a lookup that grows (`[CGP-E021]`), naming the first larger requirement,
//! and a help naming `ShowNested` as the provider whose requirement grows.
//!
//! rustc's advice to raise the recursion limit is dropped, since no limit ends this lookup; the
//! output is `acceptable`.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`); the depth cap is in
//! `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`.

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

check_components! {
    App {
        ShowerComponent: u32,
    }
}

fn main() {}
