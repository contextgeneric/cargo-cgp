//! Every consumer a use-site failure names gets its chain, sharing one root cause.
//!
//! `App` wires two independent components — `CanGreet` and `CanBidFarewell` — whose
//! providers each read the same `name` field through `HasName`, and `App` does not have
//! that field. Calling either method fails as a use-site `E0599`, and the use-site anchor
//! walks *every* component the context wires, so both fail and both are named in the
//! `[CGP-E001]` header.
//!
//! Pins that both then appear in the note. The anchor unions the two walks' causes, which
//! name one shared leaf, and `merge_causes_by_leaf` folds them into a single cause holding
//! *both* routes — where de-duplicating by leaf and discarding the duplicate's paths would
//! leave the header promising two failing consumers and the note accounting for one. The
//! two chains converge on the shared `HasName` hop, so the second `(*)`-truncates there.
//! The check-entry counterpart of this shape is `parallel_consumers`.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetWithName)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

#[cgp_component(Farewell)]
pub trait CanBidFarewell {
    fn farewell(&self) -> String;
}

#[cgp_impl(new FarewellWithName)]
#[uses(HasName)]
impl Farewell {
    fn farewell(&self) -> String {
        format!("Goodbye, {}!", self.name())
    }
}

// No `name` field, so both wired providers fail on the one shared cause.
#[derive(HasField)]
pub struct App {
    pub age: u8,
}

delegate_components! {
    App {
        GreeterComponent: GreetWithName,
        FarewellComponent: FarewellWithName,
    }
}

fn main() {
    let app = App { age: 8 };

    // No `check_components!` block, so the failure surfaces here — the use-site anchor.
    let _ = app.greet();
}
