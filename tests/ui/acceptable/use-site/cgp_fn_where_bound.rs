//! A `#[cgp_fn]` capability required through a `where` **bound**, not called as a method.
//!
//! `#[cgp_fn]` turns a function into a *blanket-impl trait* — `impl<Context> GetName for Context
//! where Self: HasField<Symbol!("name")>` — which is not a CGP component. Besides calling its method
//! directly, such a capability is commonly required as a bound on a generic function
//! (`fn greet_all<Context: GetName>(…)`), so the function works on any context providing it. When
//! that function is called with a context missing the field the capability reads, the failure is an
//! `E0277` on the *call*, and the diagnostic names the capability trait `GetName` (in the bound and
//! its definition) rather than pointing at any method call on a concrete context.
//!
//! This is the by-bound counterpart of `cgp_fn_use_site.rs` (the direct-call shape). No method call
//! on a local context sits at the failure, so the call-site anchor does not apply; the handle is the
//! capability trait the diagnostic names in its spans.
//!
//! The by-capability use-site anchor recovers it: it finds the local `#[cgp_fn]`/`#[blanket_trait]`
//! trait `GetName` named in the diagnostic's spans and the context `App` from the failing
//! expression (`app`, whose type is read off its binding — rustc puts the "not implemented for
//! `App`" span on the `#[derive(HasField)]` attribute, outside `App`'s item span, so it cannot be
//! recovered from a struct-definition span). Walking `App: GetName` reaches the missing field, and
//! because `GetName` is not a CGP component the headline reads `[CGP-E009] the trait …` over a
//! `root cause: [CGP-E106] missing field \`name\`` tree. This anchor is gated to the `E0277`
//! (capability-used-as-a-bound) shape and tried after the call-site anchor, so a direct method call
//! still leads with the capability the programmer invoked. Left to raw rustc the field name is
//! mangled to an unreadable `Symbol<4, Chars<..>>` buried in a `help`.

use cgp::prelude::*;

/// A `#[cgp_fn]` capability that reads a `name` field from its context.
#[cgp_fn]
pub fn get_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

/// A generic function that requires the capability as a `where` bound rather than calling it on a
/// concrete context.
fn greet_all<Context>(context: &Context) -> String
where
    Context: GetName,
{
    format!("Hello, {}!", context.get_name())
}

#[derive(HasField)]
pub struct App {
    // The `name` field is missing, so `App` does not satisfy `GetName`.
    pub locale: String,
}

fn run(app: &App) -> String {
    greet_all(app)
}

fn main() {}
