//! Recovering a use-site failure's obligation from the call expression's own HIR.
//!
//! This is the anchor for the use-site failure whose spans touch nothing the span-matching
//! anchors can read: the wiring matches the called component unconditionally, so the method is
//! *found* and the failure is an `E0277` whose spans never leave the call. Everything is
//! recovered from the failing call expression, HIR-only (never `tcx.typeck`, which cannot be
//! forced from the emitter): the *receiver* carries the context, the component's parameters come
//! from unifying the call's *written* argument types against the method's own declared signature
//! — no calling convention assumed — and each parameter the call leaves to inference is seeded as
//! a rigid placeholder the walk resolves around but never reports on. The failure shape, the
//! rationale for each recovery step, the worked example, and the decline boundaries are
//! documented in `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution-call-site.md`.
//!
//! The recovery is split by stage: [`find_call`] locates the failing call and the candidate
//! consumer traits, [`receiver`] reads the context off the receiver expression, [`seed`] builds
//! the obligation by signature unification, [`written_ty`] reads the types the call's arguments
//! write, and [`lower`] is the small syntactic type lowering they all stand on.

mod find_call;
mod lower;
mod receiver;
mod seed;
mod written_ty;

pub use find_call::*;
pub(crate) use lower::*;
pub(crate) use receiver::*;
pub(crate) use seed::*;
pub(crate) use written_ty::*;
