//! Usability: a *generic* consumer failing at its call site, out of reach of both
//! use-site anchors.
//!
//! The by-component anchor re-checks a generic component's bare marker with an empty
//! `()` parameter slot, which matches no real wiring, and the by-consumer anchor is
//! restricted to a consumer whose only generic is `Self` — the dispatch parameter at
//! the call (`CanFormatPair<_>`) is an inference variable no span recovers. So a
//! broken generic consumer called directly declines to the text-rewrite fallback,
//! which keeps rustc's misleading "use associated function syntax" advice ahead of
//! the real missing-field cause. Exposes issues in docs/issues/usability.md.

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
    let _ = app.format_pair((1_u32, 2_u64));
}
