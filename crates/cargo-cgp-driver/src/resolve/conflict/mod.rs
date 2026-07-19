//! Classifying a duplicate-key wiring conflict by querying the trait solver.
//!
//! A duplicate key in `delegate_components!` makes the expansion emit two overlapping
//! `DelegateComponent` impls, so the compiler reports the coherence error (`E0119`) *twice* —
//! once for the `DelegateComponent` table impl and once for the `IsProviderFor` forwarding impl
//! the same entry generates. The two are one logical mistake, so this module recognizes the
//! pair: it drops the redundant `IsProviderFor` half and rewrites the `DelegateComponent` half
//! into a message that names the colliding key(s).
//!
//! Everything is recovered from the compiler, not from the error text. The failing diagnostic's
//! primary span equals `tcx.def_span` of the *conflicting* impl (the macro re-spans each entry
//! onto its key token, and rustc aims the `E0119` at that impl's def-span), so [`classify`]
//! matches the caret to that impl and [`build`] reads the entry off it — its self type, its key,
//! and its `Delegate` — then words the collision through the key renderings in [`keys`]. Each
//! recognized trait is anchored by `DefId` exactly as the rest of [`resolve`](crate::resolve) is,
//! so a same-named trait from another crate can never drive the rewrite.

mod build;
mod classify;
mod delegate_impls;
mod keys;
mod namespace;

pub(crate) use build::*;
pub use classify::*;
pub(crate) use delegate_impls::*;
pub(crate) use keys::*;
pub(crate) use namespace::*;
