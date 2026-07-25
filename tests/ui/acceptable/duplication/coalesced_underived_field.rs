//! One underived field named once, however many coalesced consumers reach it.
//!
//! `App` declares a `name` field but no `#[derive(HasField)]`, so the field is
//! present-but-underived — one mistake with one fix. Three checked components read it
//! through a cascade (`ProvideBaz` needs `CanBar`, which needs `CanFoo`, which reads the
//! field), so all three consumer failures share that single root cause and coalesce into
//! one `[CGP-E001]` block.
//!
//! Pins that the merged block states the shared cause *once*. Coalescing several
//! **distinct** underived fields on one struct into one lead is deliberate — the derive
//! emits an impl per field, so they are one fix (`base_area_2`) — but every member here
//! contributes the **same** field, and the union of their causes therefore repeats it once
//! per member. `merge_causes_by_leaf` folds those copies back into one cause holding all
//! three paths before the underived-field coalescing runs, so the lead keeps its
//! single-field wording rather than reading "the fields `name`, `name`, and `name`", and
//! the merged tree still renders as the one subsuming chain. The cascade's missing-field
//! sibling, where the leaf never coalesces, is `dependency_cascade`.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md
//! (derive-missing variant).

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
#[uses(HasName)]
impl Foo {
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

// The field is declared, but `#[derive(HasField)]` is missing — so it is
// present-but-underived rather than absent.
pub struct App {
    pub name: String,
}

delegate_components! {
    App {
        FooComponent: ProvideFoo,
        BarComponent: ProvideBar,
        BazComponent: ProvideBaz,
    }
}

check_components! {
    App {
        BazComponent,
        BarComponent,
        FooComponent,
    }
}

fn main() {}
