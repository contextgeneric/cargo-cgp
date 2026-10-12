//! The same entry of a generic component listed twice in one `check_components!` table.
//!
//! The two check impls the table generates for `ShowerComponent: u32` are identical, so the
//! compiler rejects them with an `E0119`, which becomes `[CGP-E015]` naming the entry as written.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: acceptable
//@codes: CGP-E015

use cgp::prelude::*;

#[cgp_component(Shower)]
pub trait CanShow<T> {
    fn show(&self, value: T) -> String;
}

#[cgp_impl(new ShowDebug)]
impl<T: core::fmt::Debug> Shower<T> {
    fn show(&self, value: T) -> String {
        format!("{value:?}")
    }
}

pub struct App;

delegate_components! {
    App {
        ShowerComponent: ShowDebug,
    }
}

check_components! {
    App {
        ShowerComponent: u32,
        ShowerComponent: u32,
    }
}

fn main() {}
