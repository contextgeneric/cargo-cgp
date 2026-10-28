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
//! The recovery is split by stage: [`find_call`] locates the failing call and tries the candidate
//! consumer traits, [`receiver`] reads the context off the receiver expression, and [`seed`] builds
//! the obligation by signature unification. The types the call's arguments write, the small
//! syntactic type lowering they all stand on, and the candidate traits by method name are read in
//! the [query layer](crate::resolve::query), which the detectors share. A call to one of
//! CGP's extensible-data operations has its own anchor, [`data_call`], whose receiver is often
//! another data operation's result, typed by [`chain`]; a call to a provider trait's method through
//! the provider's own path has another, [`provider_call`]; and a failing bound of a generic
//! function, at the call that instantiates it, has a last one, [`bound_call`].

mod bound_call;
mod chain;
mod data_call;
mod find_call;
mod provider_call;
mod receiver;
mod seed;
mod unbounded;

pub use bound_call::*;
pub(crate) use chain::*;
pub use data_call::*;
pub use find_call::*;
pub use provider_call::*;
pub(crate) use receiver::*;
pub(crate) use seed::*;
pub(crate) use unbounded::*;
