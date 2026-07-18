//! A component whose single generic parameter is instantiated with a *tuple* type.
//!
//! CGP's params encoding is ambiguous here: a check entry `(u32, u64)` is the same
//! params tuple whether the component has two parameters or one tuple-typed one.
//! The resolver rebuilds the consumer obligation from that slot, so it must not
//! mistake the one tuple-typed parameter for two separate ones.

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

check_components! {
    App {
        PairFormatterComponent: (u32, u64),
    }
}

fn main() {}
