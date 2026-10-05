//! A `#[cgp_impl]` provider body that calls a trait method without declaring the trait via
//! `#[uses]`: the provider counterpart of [`undeclared_uses_trait`](undeclared_uses_trait.rs).
//!
//! `#[cgp_impl]` lowers the provider into `impl<__Context__> Greeter<__Context__> for GreetHello`,
//! renaming `self` to a `__context__: &__Context__` parameter. A trait the body calls on `self` must
//! be a `where` bound on `__Context__`, declared with `#[uses(…)]`. Here the body calls
//! `self.person_name()`, a `#[cgp_fn]` trait, and declares nothing, so the call cannot resolve.
//!
//! Raw rustc reports an `E0599` saying the method exists for `&__Context__` but its trait bounds
//! were not satisfied, naming the generated `__Context__` and pointing at a `HasField` bound of the
//! called trait rather than at the missing `#[uses(PersonName)]`. Unlike the `#[cgp_fn]` case, the
//! generated impl's `Self` is the provider struct, not the bare `__Context__` parameter, so the
//! failing call's receiver, a parameter declared as `&__Context__`, is what identifies the shape.
//!
//! The tool reshapes it into the same `[CGP-E012]` error as the `#[cgp_fn]` case, with a
//! `help: declare it as a dependency with `#[uses(PersonName)]``, recovered by
//! `resolve::detect_undeclared_trait` from the receiver's declared type. CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.

use cgp::prelude::*;

#[cgp_fn]
fn person_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.person_name())
    }
}

fn main() {}
