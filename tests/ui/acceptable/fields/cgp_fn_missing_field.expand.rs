#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
/// A `#[cgp_fn]` capability that reads a `name` field from its context.
pub trait FormatName {
    fn format_name(&self) -> String;
}
/// A `#[cgp_fn]` capability that reads a `name` field from its context.
impl<__Context__> FormatName for __Context__
where
    Self: HasField<Symbol!("name"), Value = String>,
{
    fn format_name(&self) -> String {
        let name: &str = self
            .get_field(::core::marker::PhantomData::<Symbol!("name")>)
            .as_str();
        name.to_owned()
    }
}
/// A second `#[cgp_fn]` capability that composes the first through `#[uses]`, so its blanket impl
/// depends on `Self: FormatName` rather than on a field directly.
pub trait Greeting {
    fn greeting(&self) -> String;
}
/// A second `#[cgp_fn]` capability that composes the first through `#[uses]`, so its blanket impl
/// depends on `Self: FormatName` rather than on a field directly.
impl<__Context__> Greeting for __Context__
where
    Self: FormatName,
{
    fn greeting(&self) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Hello, {0}!", self.format_name()))
        })
    }
}
pub struct App {
    pub locale: String,
}
impl HasField<Symbol!("locale")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("locale")>,
    ) -> &Self::Value {
        &self.locale
    }
}
impl HasFieldMut<Symbol!("locale")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("locale")>,
    ) -> &mut Self::Value {
        &mut self.locale
    }
}
pub trait CheckFormatName: FormatName {}
impl CheckFormatName for App {}
pub trait CheckGreeting: Greeting {}
impl CheckGreeting for App {}
fn main() {}
