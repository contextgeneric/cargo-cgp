//! A `#[cgp_fn]` trait required by a **struct's** bound, built with a context missing the field the
//! trait reads.
//!
//! `#[cgp_fn]` turns `get_name` into the blanket-impl trait `GetName`, which any type with a `name`
//! field implements. `Holder<T: GetName>` requires it of what it holds, so building `Holder(app)`
//! for an `App` without that field fails with an `E0277` on the argument, `required by a bound in
//! `Holder``. The root cause is the missing `name` field on `App`.
//!
//! rustc's note lands on the bound inside `Holder`'s generics, so `Holder` is only passing the
//! requirement on, and the context is the argument the caret points at.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E003 CGP-E104 CGP-E110

use cgp::prelude::*;

#[cgp_fn]
fn get_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

pub struct Holder<T: GetName>(pub T);

#[derive(HasField)]
pub struct App {
    pub label: String,
}

fn main() {
    let app = App { label: String::new() };
    let _holder = Holder(app);
}
