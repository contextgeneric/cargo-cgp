//! A `#[cgp_fn]` whose generic output is pinned by a provider bound, the shape of a JSON helper.
//! The call leaves the output to inference, so the call-site anchor seeds it as a placeholder and
//! still reaches the provider's missing field.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.

//@quality: acceptable
//@codes: CGP-E003 CGP-E102 CGP-E104 CGP-E110

use cgp::prelude::*;

#[cgp_component(Loader)]
pub trait CanLoad<T> {
    type Output;
    fn load(&self) -> Self::Output;
}

#[cgp_new_provider]
impl<Context, T> Loader<Context, T> for LoadVia
where
    T: Default,
    Context: HasName,
{
    type Output = T;
    fn load(context: &Context) -> T {
        let _ = context.name();
        T::default()
    }
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_fn(CanFetchValue)]
pub fn fetch_value<T>(&self) -> T
where
    LoadVia: Loader<Self, T, Output = T>,
{
    LoadVia::load(self)
}

#[derive(HasField)]
pub struct App {
    pub other: String,
}

fn main() {
    let app = App { other: String::new() };
    let _: u32 = app.fetch_value();
}
