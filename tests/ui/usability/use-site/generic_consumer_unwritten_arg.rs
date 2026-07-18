//! Usability: a generic consumer failing at a call whose arguments write no types.
//!
//! The dispatch parameter is carried by a plain variable argument, so nothing at the
//! call names its type: the span-matching anchors cannot recover it (the bare-marker
//! re-check uses an empty `()` slot, the by-consumer anchor needs a `Self`-only
//! consumer), and the call-site anchor's signature unification has no written
//! argument type to consume — typing `pair` would need the typeck results the
//! emitter can never force. The parameter is seeded as an unknown, every root cause
//! behind the wiring depends on it, and the resolver declines to the text-rewrite
//! fallback, which keeps rustc's misleading "use associated function syntax" advice
//! ahead of the buried missing-field cause. Exposes issues in
//! docs/issues/usability.md.

use cgp::prelude::*;

#[cgp_component(PairFormatter)]
pub trait CanFormatPair<T> {
    fn format_pair(&self, value: T) -> String;
}

#[cgp_auto_getter]
pub trait HasSeparator {
    fn separator(&self) -> &String;
}

#[cgp_impl(new FormatWithSeparator)]
impl PairFormatter<(u32, u64)>
where
    Self: HasSeparator,
{
    fn format_pair(&self, value: (u32, u64)) -> String {
        format!("{}{}{}", value.0, self.separator(), value.1)
    }
}

#[derive(HasField)]
pub struct App {
    // missing `separator` field to trigger the error
    pub dummy: (),
}

delegate_components! {
    App {
        PairFormatterComponent: FormatWithSeparator,
    }
}

fn main() {
    let app = App { dummy: () };
    let pair = (1_u32, 2_u64);
    let _ = app.format_pair(pair);
}
