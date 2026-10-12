//! One wiring mistake reported once. CGP wiring is lazy, so the single missing `name` field
//! `GreetHello` needs fans out across the `check_components!` entry and the hand-written
//! `CanGreetSend` impl (both its header and its forwarding `self.greet()` call), the `Send`-recovery
//! shape of a wrapper trait over a CGP consumer. The re-reports of each failing trait collapse first,
//! and the two failing traits then share their root cause, so they coalesce into one block headed by
//! that cause, with a caret at the check entry and at the impl. The check entry comes first in the
//! source, so the block takes its `[CGP-E001]` code. Its tree is headed by `CanGreetSend` (the code
//! the programmer wrote), descending through its `CanGreet` supertrait to the missing field, so the
//! wrapper's route and the check's route are one chain. The `.rust.stderr` baseline shows the full
//! cascade for contrast.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        let _ = self.name();
    }
}

// The single mistake: `App` never carries the `name` field `GreetHello` reads.
pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

// Site 1: the wiring check.
check_components! {
    App {
        GreeterComponent,
    }
}

// Sites 2 and 3: a hand-written wrapper trait carrying `CanGreet` as a supertrait, implemented
// directly on the context to add a `Send` bound the component cannot express — failing at the impl
// header and again at the forwarding call, both the same failure as the check above.
pub trait CanGreetSend: CanGreet + Send {
    fn greet_send(&self);
}

impl CanGreetSend for App {
    fn greet_send(&self) {
        self.greet()
    }
}

fn main() {}
