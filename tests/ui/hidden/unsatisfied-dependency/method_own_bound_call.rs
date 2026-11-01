//! A method whose own `where` clause requires a consumer the context cannot satisfy. The bound-call
//! anchor reads a path call's function, not a method's bound, and `CanFetch` shows no CGP evidence of
//! its own, so the resolver declines and rustc's headline stays over the `missing field` help.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Loader)]
pub trait CanLoad<T> {
    type Output;
    fn load(&self) -> Self::Output;
}

#[cgp_impl(new LoadName)]
impl<T> Loader<T>
where
    T: Default,
{
    type Output = T;
    fn load(&self, #[implicit] name: &str) -> T {
        let _ = name;
        T::default()
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

#[derive(HasField)]
pub struct App {
    pub other: String,
}

delegate_components! {
    App {
        LoaderComponent: LoadName,
    }
}

fn main() {
    let app = App { other: String::new() };
    let _: u32 = app.fetch::<u32>();
}
