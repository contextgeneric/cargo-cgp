//! Acceptable failure: a field the context reaches through `Deref`, whose `Deref` target does
//! not derive `HasField`. CGP's `HasField` follows `Deref` (a blanket impl forwards to the
//! target), so `App` *would* satisfy `HasField<Symbol!("name")>` if its `Deref` target
//! `AppFields` derived it — but `AppFields` deliberately omits `#[derive(HasField)]`, so the
//! forward has nothing to reach and the `GreetHello` wiring fails.
//!
//! This fixture pins the driver's `Deref`-aware diagnosis: rather than reporting `name` as a
//! plain missing field (it is not — `AppFields` carries it), the resolver walks `App`'s `Deref`
//! chain, finds `name` on `AppFields`, and points the fix at the type that must derive
//! `HasField` — `AppFields`, not `App`.
//!
//! See docs/errors/checks/check-trait-failure.md and
//! docs/implementation/typed-root-cause-resolution.md (the field-inspection variants).

use core::ops::Deref;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) {
        let _ = self.name();
    }
}

// `AppFields` carries `name` but deliberately omits `#[derive(HasField)]`, so `App`'s
// `Deref`-forwarded `HasField` impl has no target impl to reach.
pub struct AppFields {
    pub name: String,
}

// `App` reaches `name` only through `Deref` to `AppFields`.
pub struct App {
    pub fields: AppFields,
}

impl Deref for App {
    type Target = AppFields;

    fn deref(&self) -> &AppFields {
        &self.fields
    }
}

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
