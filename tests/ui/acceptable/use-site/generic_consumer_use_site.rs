//! A *generic* consumer failing at its call site, its parameter recovered from a
//! plain value argument.
//!
//! The span-matching use-site anchors cannot reach this: the by-component anchor
//! re-checks the bare marker with an empty `()` parameter slot, and the by-consumer
//! anchor is restricted to a consumer whose only generic is `Self`. The call-site
//! anchor recovers it instead, with no calling convention assumed: the context comes
//! from the receiver's `let` binding, and unifying the written argument type — the
//! suffixed-literal tuple `(1_u32, 2_u64)` — against the method's declared
//! `value: T` input pins `T = (u32, u64)` through the signature alone. The seeded
//! `App: CanFormatPair<(u32, u64)>` then walks to the missing `separator` field,
//! and rustc's misleading "use associated function syntax" advice is dropped with
//! the rest of its sub-notes.

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
