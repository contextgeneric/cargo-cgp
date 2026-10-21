//! A wiring whose lookup grows in its `Self` type: `ShowWrapped` implements `Shower` for a context
//! by requiring `CanShow` of `Vec<Self>`, and both `u32` and every `Vec<T>` are wired to it, so
//! checking `u32` asks for `Vec<u32>: CanShow`, then `Vec<Vec<u32>>: CanShow`, and so on, until the
//! trait solver overflows (`E0275`).
//!
//! The tool's walk re-roots at each `Vec<…>`, a context since it carries wiring of its own, so it
//! descends the growing requirements until its depth cap cuts the lookup. The overflow reads as
//! `[CGP-E021]`, naming the first larger type, `Vec<u32>`, and the help names `ShowWrapped`, the
//! provider that requires `CanShow` of it.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`).

//@quality: acceptable
//@codes: CGP-E021

use cgp::prelude::*;

#[cgp_component(Shower)]
pub trait CanShow {
    fn show(&self) -> String;
}

#[cgp_impl(new ShowWrapped)]
impl Shower
where
    Vec<Self>: CanShow,
{
    fn show(&self) -> String {
        String::new()
    }
}

delegate_components! {
    u32 {
        ShowerComponent: ShowWrapped,
    }
}

delegate_components! {
    <T> Vec<T> {
        ShowerComponent: ShowWrapped,
    }
}

check_components! {
    u32 {
        ShowerComponent,
    }
}

fn main() {}
