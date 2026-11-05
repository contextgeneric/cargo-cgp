//! A `#[uses]` dependency that gives its consumer trait one generic argument too many.
//!
//! `CanFoo<T>` takes one argument, and `#[uses(CanFoo<u32, u64>)]` gives it two. The error is a
//! plain arity slip (`E0107`), and rustc's message names it with the fix. The bound sits on the
//! generated context, not on an inner provider, so it is no consumer trait in an inner-provider
//! bound and must stay exactly as rustc prints it.
//!
//! CGP error class: none; plain Rust the tool passes through.

use cgp::prelude::*;

#[cgp_component(Fooer)]
pub trait CanFoo<T> {
    fn foo(&self, value: T) -> u32;
}

#[cgp_component(Barer)]
pub trait CanBar {
    fn bar(&self) -> u32;
}

// MISTAKE: `CanFoo` takes one generic argument, and the dependency gives it two.
#[cgp_impl(new BarFromFoo)]
#[uses(CanFoo<u32, u64>)]
impl Barer {
    fn bar(&self) -> u32 {
        self.foo(1)
    }
}

fn main() {}
