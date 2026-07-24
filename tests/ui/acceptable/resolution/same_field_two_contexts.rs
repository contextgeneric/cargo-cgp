//! Two genuinely distinct root causes that happen to *name* the same thing, kept apart. `Outer`'s
//! provider reads a `name` field from its own context and also depends on the concrete `Inner`
//! context being able to compute, whose provider reads a `name` field of *its* own. Neither struct
//! carries one, so `Outer`'s dependency tree bottoms out on two missing fields that share a name but
//! sit on different structs — and need two separate fixes.
//!
//! Causes are grouped by whole-leaf equality, so the two stay distinct and the note heads a
//! `root causes:` list naming both owners. Grouping them by the field name alone would merge them
//! into one cause whose singular heading named only the first struct, while the tree below it still
//! branched to both — a heading that understates the work.
//!
//! The cross-context machinery this leans on (the `where Inner: CanCompute` recovery, the re-rooting
//! that makes `Inner`'s subtree decode against `Inner`) is pinned separately by
//! [`cross_context_node_key`](cross_context_node_key.rs); here it is only the shortest way to reach
//! two same-named fields on different owners from one check entry.

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

// Two dependencies, each reaching a `name` field on a different struct: `Self: HasName` on the
// checked context, and `Inner: CanCompute` on the other context, whose own provider needs `Inner`'s.
#[cgp_impl(new RunViaInner)]
#[uses(HasName)]
impl Runner
where
    Inner: CanCompute,
{
    fn run(&self) {
        let _ = self.name();
    }
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
    Outer {
        RunnerComponent,
    }
}

fn main() {}
