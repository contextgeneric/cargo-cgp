//! A failing call whose receiver is a `let` with a type annotation.
//!
//! `make_app()` is the initializer, but the annotation is what names the context: the call-site
//! anchor reads `App` off `let app: App`, seeds `App: CanFormatPair<(u32, u64)>`, and walks to the
//! missing `separator` field.

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
    pub dummy: (),
}

delegate_components! {
    App {
        PairFormatterComponent: FormatWithSeparator,
    }
}

fn make_app<T: From<App>>() -> T {
    App { dummy: () }.into()
}

fn main() {
    let app: App = make_app();
    let _ = app.format_pair((1_u32, 2_u64));
}
