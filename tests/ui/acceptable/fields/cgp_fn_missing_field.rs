//! A `#[cgp_fn]` capability check that fails because the context is missing the field the
//! capability reads. `#[cgp_fn]` turns a function into a *blanket-impl trait* — `impl<Context>
//! FormatName for Context where Self: HasField<Symbol!("name"), Value = String>` — which is not a
//! CGP component (there is no provider trait or `DelegateComponent`). A common way to assert such a
//! capability holds for a concrete context is a wrapper trait carrying it as a supertrait,
//! implemented on the context: `pub trait CheckFormatName: FormatName {}` + `impl CheckFormatName
//! for App {}`. When the context lacks the field, that impl fails with an `E0277`.
//!
//! The impl-site anchor recognizes the `impl CheckFormatName for App` block and its failing
//! supertrait on the local context, and walks it whether that supertrait is a CGP *component*
//! consumer or a `#[cgp_fn]`/`#[blanket_trait]` blanket-impl trait, so the failure resolves to the
//! real cause — `` missing field `name` on `App` `` — through the `FormatName` (and, for `greeting`,
//! the `#[uses]`-chained `Greeting → FormatName`) blanket-trait chain. Left to raw rustc the
//! `` `#[derive(HasField)]` is required to access field `name` `` help is actively misleading here:
//! `App` *does* derive `HasField`; it is the `name` *field* that is absent (rustc then lists the
//! fields it does have).

use cgp::prelude::*;

/// A `#[cgp_fn]` capability that reads a `name` field from its context.
#[cgp_fn]
pub fn format_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

/// A second `#[cgp_fn]` capability that composes the first through `#[uses]`, so its blanket impl
/// depends on `Self: FormatName` rather than on a field directly.
#[cgp_fn]
#[uses(FormatName)]
pub fn greeting(&self) -> String {
    format!("Hello, {}!", self.format_name())
}

#[derive(HasField)]
pub struct App {
    // No `name` field — the `#[cgp_fn]` capabilities above cannot be satisfied.
    pub locale: String,
}

pub trait CheckFormatName: FormatName {}
impl CheckFormatName for App {}

pub trait CheckGreeting: Greeting {}
impl CheckGreeting for App {}

fn main() {}
