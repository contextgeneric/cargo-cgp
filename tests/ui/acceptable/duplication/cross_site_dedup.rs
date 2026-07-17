//! Acceptable: one wiring mistake, reported once. CGP wiring is lazy, so a single missing
//! dependency surfaces wherever the consumer trait is used. Here `App` lacks the `name` field
//! `GreetHello` needs, and that one mistake would otherwise produce three separate errors: the
//! `check_components!` entry, the hand-written `CanGreetSend` impl header (the transfer example's
//! `CanHandleApiSend` shape — a wrapper supertrait implemented directly on the context), and its
//! forwarding `self.greet()` call. All three recover the same consumer trait (`CanGreet`) and the
//! same root cause (missing field `name` on `App`), so the emitter de-duplicates them by recovered
//! cause and shows only the first. The committed `.rust.stderr` baseline shows the full
//! un-deduplicated cascade for contrast.
//!
//! Two distinct consumers that happened to share a cause would *not* merge (the signature includes
//! the consumer), so no distinct capability's failure is ever hidden.

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
