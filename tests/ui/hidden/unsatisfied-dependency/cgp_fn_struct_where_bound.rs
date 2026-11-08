//! A `#[cgp_fn]` trait required by a struct's `where` clause, built with a context missing the field
//! the trait reads. The root cause is the missing `name` field on `App`, and `Holder` only passes the
//! requirement on.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: hidden
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
fn get_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

pub struct Holder<T>(pub T)
where
    T: GetName;

#[derive(HasField)]
pub struct App {
    pub label: String,
}

fn main() {
    let app = App { label: String::new() };
    let _holder = Holder(app);
}
