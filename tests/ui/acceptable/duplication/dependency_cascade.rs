//! One root cause reported once across a chain of dependent components.
//!
//! `ProvideFoo` needs the `name` field, `ProvideBar` depends on `CanFoo`, and
//! `ProvideBaz` depends on `CanBar`; wiring all three onto an `App` without a `name`
//! field and checking all three would, left to rustc, cascade into a block per
//! component — more than three, since the deeper providers also emit intermediate
//! provider-bound failures. cargo-cgp coalesces them: the three consumer failures
//! share the one missing-`name` root cause, so they collapse into a single
//! `[CGP-E001]` headline naming `CanBaz`, `CanBar`, and `CanFoo`, a caret at each check
//! entry, and one representative dependency chain down to the missing field (the
//! first-checked `CanBaz`, whose chain is the deepest and subsumes the others). Fixing
//! the one field clears the whole cascade. One entry surfaces to rustc as a
//! provider-side bound, but coalescing words the group uniformly as consumer traits,
//! since a `check_components!` entry failing *is* the consumer trait failing.
//!
//! See docs/errors/checks/verbose-cascade.md.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Foo)]
pub trait CanFoo {
    fn foo(&self);
}

#[cgp_component(Bar)]
pub trait CanBar {
    fn bar(&self);
}

#[cgp_component(Baz)]
pub trait CanBaz {
    fn baz(&self);
}

#[cgp_impl(new ProvideFoo)]
impl Foo
where
    Self: HasName,
{
    fn foo(&self) {
        let _ = self.name();
    }
}

#[cgp_impl(new ProvideBar)]
#[uses(CanFoo)]
impl Bar {
    fn bar(&self) {
        self.foo();
    }
}

#[cgp_impl(new ProvideBaz)]
#[uses(CanBar)]
impl Baz {
    fn baz(&self) {
        self.bar();
    }
}

#[derive(HasField)]
pub struct App {
    pub age: u8,
}

delegate_components! {
    App {
        FooComponent: ProvideFoo,
        BarComponent: ProvideBar,
        BazComponent: ProvideBaz,
    }
}

// Each checked component fails because `App` lacks the `name` field the innermost
// provider needs, producing one error block per component. Listed top-down, so the
// root-cause block (`Foo`, naming the missing field) is reported last.
check_components! {
    App {
        BazComponent,
        BarComponent,
        FooComponent,
    }
}

fn main() {}
