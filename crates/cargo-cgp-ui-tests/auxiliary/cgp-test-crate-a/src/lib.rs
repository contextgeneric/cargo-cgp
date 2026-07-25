//! Upstream crate for cross-crate CGP coherence and orphan-rule UI fixtures.
//!
//! This crate defines a getter capability, a component with a provider, a
//! component registered under the `@app` namespace, and a shared namespace that
//! downstream code joins. A fixture pulls it in with a `//@aux-build:
//! cgp-test-crate-a` header directive; the orphan-rule fixtures then try to
//! register impls against these foreign items and are rejected by the compiler,
//! while the positive `ok/` fixture (through `cgp-test-crate-b`) wires them
//! successfully — exercising that CGP's two-trait split stays within Rust's
//! coherence and orphan rules across crate boundaries.
//!
//! See the CGP coherence concept:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/coherence.md>.

use cgp::prelude::*;

/// A published field accessor. Any context with a `name` field gains it through
/// the blanket `#[cgp_auto_getter]` impl, with no wiring required.
#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

/// A component whose provider a downstream context can wire — or replace with its
/// own provider.
#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

/// A component registered under the `@app` namespace. A downstream context wires
/// it through `delegate_components! { … namespace DefaultNamespace; @app.…: … }`.
#[cgp_component(Announcer)]
#[prefix(@app in DefaultNamespace)]
pub trait CanAnnounce {
    fn announce(&self) -> String;
}

#[cgp_impl(new AnnounceLoudly)]
impl Announcer
where
    Self: HasName,
{
    fn announce(&self) -> String {
        format!("ANNOUNCEMENT from {}!", self.name())
    }
}

// A shared namespace downstream crates populate and join. It inherits the
// built-in `DefaultNamespace`, so a context joining it also inherits the standard
// defaults. `cgp-test-crate-b` registers a *local* component into it with
// `#[default_impl]` — orphan-safe because the crate owns the component key even
// though it does not own this namespace.
cgp_namespace! {
    new AppNamespace: DefaultNamespace {}
}

/// A published `#[cgp_fn]` **capability** — a blanket-impl trait rather than a component,
/// composed from the getter above so its blanket depends on `Self: HasName` and, through
/// that, on a `HasField`. Downstream code calls `app.describe()` directly, which is how a
/// capability is normally consumed.
///
/// It exists so a fixture can exercise a capability the checked crate does **not** define.
/// Recognizing one is what earns the `[CGP-E009]` reshaping, and a blanket impl alone is far
/// too broad a signal to key on (`ToString` and `Into` have one), so recognition needs
/// positive CGP evidence for a foreign trait — which this capability's `HasName`/`HasField`
/// chain supplies.
#[cgp_fn]
#[uses(HasName)]
pub fn describe(&self) -> String {
    format!("<{}>", self.name())
}
