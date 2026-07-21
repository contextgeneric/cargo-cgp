//! A `#[cgp_fn]` capability called at a **use site**, on a context that cannot satisfy it.
//!
//! `#[cgp_fn]` turns a function into a *blanket-impl trait* — `impl<Context> Describe for Context
//! where Self: GetName + GetCount` — which is not a CGP component (there is no provider trait or
//! `DelegateComponent`). Such a capability is most naturally exercised by simply calling its method
//! on a value: `app.describe()`. When the context is missing a field one of the composed
//! capabilities reads, CGP's lazy wiring lets the call type-check far enough that the failure
//! surfaces as an `E0599` "the method exists … but its trait bounds were not satisfied", with the
//! real cause — the absent field — buried in a mid-stack note and drowned under rustc's method-probe
//! candidate list.
//!
//! This is the use-site counterpart of the impl-site `cgp_fn_missing_field` case
//! (`acceptable/fields/cgp_fn_missing_field.rs`), which asserts the same capability through a
//! wrapper `impl` block. Here there is no wrapper and no check — the capability is called directly,
//! so no `impl`- or check-site span exists to anchor on, and the only handle is the call expression
//! itself.
//!
//! The [call-site anchor](../../../docs/implementation/typed-resolution-call-site.md) recovers it:
//! it reads the receiver's context (`App`, off the `describe_app` parameter) and the called method
//! name (`describe`), finds the local blanket-impl trait `Describe` declaring that method, and walks
//! `App: Describe` to the missing field. Because a `#[cgp_fn]` capability is not a CGP *component*,
//! the headline reads `[CGP-E009] the trait …` (as the impl-site anchor words such a trait), over a
//! `root cause: [CGP-E106] missing field \`name\`` tree — the `GetName` branch that fails, with the
//! holding `GetCount` branch omitted. Left to raw rustc this is an `E0599` whose real cause is a
//! mid-stack note buried under a method-probe candidate list.

use cgp::prelude::*;

/// A `#[cgp_fn]` capability that reads a `name` field from its context.
#[cgp_fn]
pub fn get_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

/// A second `#[cgp_fn]` capability that reads a `count` field from its context.
#[cgp_fn]
pub fn get_count(&self, #[implicit] count: &u64) -> u64 {
    *count
}

/// A composite `#[cgp_fn]` capability that uses both, so its blanket impl depends on
/// `Self: GetName + GetCount` rather than on a field directly.
#[cgp_fn]
#[uses(GetName, GetCount)]
pub fn describe(&self) -> String {
    format!("{} ({})", self.get_name(), self.get_count())
}

#[derive(HasField)]
pub struct App {
    // The `name` field is missing, so `GetName` — and therefore `Describe` — cannot be satisfied.
    // `count` is present, so `GetCount` holds; only the one branch fails.
    pub count: u64,
}

fn describe_app(app: &App) -> String {
    app.describe()
}

fn main() {}
