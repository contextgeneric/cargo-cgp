//! A generic function's bound on a generic consumer trait, its context from a binding of reference
//! type.
//!
//! `format_it(app)` fills the declared input `&Context`, and `app` is declared `&App`, so the
//! bound-call anchor needs the binding's type with its reference to pin `Context = App`. It then
//! seeds the bound `App: CanFormatPair<u32>` and walks to the missing `separator` field. The consumer
//! takes a parameter, so no other anchor recovers this bound from the diagnostic's spans.

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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
impl PairFormatter<u32>
where
    Self: HasSeparator,
{
    fn format_pair(&self, value: u32) -> String {
        format!("{}{}", self.separator(), value)
    }
}

#[derive(HasField)]
pub struct App {
    pub dummy: (),
}

delegate_components! {
    App {
        PairFormatterComponent: FormatWithSeparator,
    }
}

fn format_it<Context: CanFormatPair<u32>>(context: &Context) -> String {
    context.format_pair(1)
}

fn run(app: &App) -> String {
    format_it(app)
}

fn main() {}
