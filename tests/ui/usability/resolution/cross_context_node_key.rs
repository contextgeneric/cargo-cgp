//! Still-problematic: a cross-context dependency, where one context's wiring depends on a *concrete*
//! other context. `Inner` wires and checks its own `CanCompute` component, whose provider needs a
//! `name` field `Inner` lacks. `Outer` wires a `CanRun` provider whose `where Inner: CanCompute`
//! clause depends on `Inner`, so the obligation `Inner: CanCompute` sits inside `Outer`'s dependency
//! tree while also being `Inner`'s own checked component — the shape that makes the resolution
//! cache's per-context node key load-bearing.
//!
//! The one part that works is the context distinction the cache key exists for: the shared
//! `Inner: CanCompute` node renders as a `[CGP-E101] consumer trait impl … for context \`Inner\`` in
//! `Inner`'s own tree (where `Inner` is the root context) but as a plain `trait impl … for \`Inner\``
//! in `Outer`'s tree (where `Outer` is the root), because `Outer`'s walk re-resolves the node under
//! its own context rather than reusing `Inner`'s cached subtree. Were the cache keyed on the
//! obligation alone, one tree would splice in the other's context-specific labels.
//!
//! What remains problematic is the presentation around that node, and it is why this fixture is a
//! usability case rather than an acceptable one:
//!   - The failure that anchors on `RunViaInner`'s own `where` clause resolves through the impl-site
//!     anchor onto the *provider* impl, so its dependency tree leaks the reserved `__Context__`
//!     placeholder and an `IsProviderFor` hop — internals the resolver otherwise never prints. The
//!     documented resolver boundary is that a caret on a provider struct's own impl should decline;
//!     this concrete-context `where` clause slips past it.
//!   - `Outer`'s own tree descends past `Inner: CanCompute` into `Inner`'s provider chain and bottoms
//!     out on `[CGP-E201] Inner: HasName` (an ordinary bound) with a spurious `for provider \`Inner\``
//!     hop, rather than the decoded `[CGP-E106] missing field \`name\`` the sibling blocks show.
//!
//! Recovering these — declining the provider-impl anchor and decoding the foreign-context field leaf
//! — is the work here. See docs/implementation/cached-dependency-resolution.md (the cache key) and
//! docs/implementation/typed-root-cause-resolution.md (the provider-impl anchor boundary).

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
