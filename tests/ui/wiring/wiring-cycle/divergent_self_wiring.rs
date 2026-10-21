//! A wiring whose lookup grows in its `Self` type: `ShowWrapped` implements `Shower` for a context
//! by requiring `CanShow` of `Vec<Self>`, and both `u32` and every `Vec<T>` are wired to it, so
//! checking `u32` asks for `Vec<u32>: CanShow`, then `Vec<Vec<u32>>: CanShow`, and so on, until the
//! trait solver overflows (`E0275`).
//!
//! The tool's walk does not see the growth: it cannot descend `Vec<u32>: CanShow`, whose own
//! evaluation overflows, so it stops there and treats that bound as an ordinary one left unmet. The
//! overflow keeps rustc's header, but its notes are replaced by a resolved note whose root cause is
//! ``the trait bound `Vec<u32>: CanShow` is not satisfied``, which is misleading: `Vec<u32>` is
//! wired, and the real cause is a lookup that grows. The output is `usability`; the walk should cut
//! it as a growing lookup (`[CGP-E021]`), or leave rustc's notes in place.
//!
//! The error class is the wiring cycle
//! (`cgp-knowledge-base/cgp/errors/wiring/wiring-cycle.md`).

//@quality: usability
//@codes: CGP-E101 CGP-E102 CGP-E117

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
