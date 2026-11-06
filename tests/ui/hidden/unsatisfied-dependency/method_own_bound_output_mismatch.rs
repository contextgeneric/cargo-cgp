//! The same method bound failing only on its associated type. No anchor reads it, so rustc's
//! `E0271` stays as written.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Loader)]
pub trait CanLoad<T> {
    type Output;
    fn load(&self) -> Self::Output;
}

#[cgp_impl(new LoadString)]
impl<T> Loader<T> {
    type Output = String;
    fn load(&self) -> String {
        String::new()
    }
}

pub trait CanFetch {
    fn fetch<T>(&self) -> T
    where
        Self: CanLoad<T, Output = T>;
}

impl<C> CanFetch for C {
    fn fetch<T>(&self) -> T
    where
        Self: CanLoad<T, Output = T>,
    {
        self.load()
    }
}

pub struct App;

delegate_components! {
    App {
        LoaderComponent: LoadString,
    }
}

fn main() {
    let app = App;
    let _: u32 = app.fetch::<u32>();
}
