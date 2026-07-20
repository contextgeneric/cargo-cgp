//! A cross-context dependency — one context's wiring depending on a *concrete* other context —
//! resolved cleanly, and the shape that makes the resolution cache's per-context node key
//! load-bearing. `Inner` wires and checks its own `CanCompute` component, whose provider needs a
//! `name` field `Inner` lacks. `Outer` wires a `CanRun` provider whose `where Inner: CanCompute`
//! clause depends on `Inner`, so the obligation `Inner: CanCompute` appears in both dependency trees
//! — as `Inner`'s own checked consumer (root context `Inner`) and as an interior node of `Outer`'s
//! tree (root context `Outer`).
//!
//! Three behaviors combine here:
//!   - The provider impl's own `where Inner: CanCompute` clause is recovered as the consumer
//!     obligation it is (the impl-site anchor reads the concrete-context bound directly rather than
//!     declining on the provider impl), so it de-duplicates into `Inner`'s own `[CGP-E001]` block
//!     rather than leaving rustc's raw bound error — the two sites of one mistake collapse to one.
//!   - `Outer`'s tree re-roots the `Inner: CanCompute` node at `Inner` while walking, so it decodes
//!     to `[CGP-E106] missing field \`name\`` (not an opaque bound), its delegation-routing hop is
//!     dropped, and it reads as a consumer node `for context Inner`.
//!   - The node renders as a `[CGP-E101] consumer trait impl … for context \`Inner\`` in both trees
//!     precisely because the cache keys each node on `(obligation, context)`: `Outer`'s walk re-roots
//!     to `Inner` and thereby shares `Inner`'s own cached subtree, rather than borrowing `Outer`'s
//!     context. Were the key the obligation alone, one tree would splice in the other's labels.
//!
//! See docs/implementation/cached-dependency-resolution.md (the cache key) and
//! docs/implementation/typed-root-cause-resolution.md (the impl-site anchor and cross-context walk).

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Computer)]
pub trait CanCompute {
    fn compute(&self);
}

#[cgp_impl(new DoCompute)]
#[uses(HasName)]
impl Computer {
    fn compute(&self) {
        let _ = self.name();
    }
}

#[cgp_component(Runner)]
pub trait CanRun {
    fn run(&self);
}

// Cross-context dependency: `Outer`'s runner requires the concrete `Inner` context to be able to
// compute, so `Inner: CanCompute` sits inside `Outer`'s dependency tree while also being `Inner`'s
// own checked component.
#[cgp_impl(new RunViaInner)]
impl Runner
where
    Inner: CanCompute,
{
    fn run(&self) {}
}

#[derive(HasField)]
pub struct Inner {
    pub age: u8,
}

#[derive(HasField)]
pub struct Outer {
    pub label: u8,
}

delegate_components! {
    Inner {
        ComputerComponent: DoCompute,
    }
}

delegate_components! {
    Outer {
        RunnerComponent: RunViaInner,
    }
}

check_components! {
    Inner {
        ComputerComponent,
    }
}

check_components! {
    Outer {
        RunnerComponent,
    }
}

fn main() {}
