//! Acceptable: one wiring mistake, its re-reports collapsed per failing trait, each error headed by
//! the code the programmer wrote. CGP wiring is lazy, so the single missing `name` field `GreetHello`
//! needs fans out across the `check_components!` entry and the hand-written `CanGreetSend` impl (both
//! its header and its forwarding `self.greet()` call) — the transfer example's `Send`-recovery shape.
//! The tool collapses the re-reports of each failing trait to one block: the check entry becomes a
//! `[CGP-E001]` `CanGreet` consumer-trait error, and the `CanGreetSend` impl (header and call) a
//! single `[CGP-E009]` error. A wrapper is a plain trait, not a CGP consumer, so its header reads
//! "the trait", and its tree is headed by `CanGreetSend` (the code the programmer wrote), descending
//! through its `CanGreet` supertrait to the missing field rather than hiding the wrapper. The two
//! blocks stay distinct because they are distinct traits; no re-report repeats. The `.rust.stderr`
//! baseline shows the full cascade for contrast.

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
