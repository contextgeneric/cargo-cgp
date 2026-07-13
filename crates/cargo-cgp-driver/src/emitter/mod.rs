//! The diagnostic-transforming emitter the driver installs.
//!
//! This is the compiler-side seam of every diagnostic transform. It replaces the session's own
//! emitter with [`CgpEmitter`], which acts on each diagnostic before handing it to a real inner
//! emitter, so the transformed result reaches cargo already shaped and rendered like vanilla
//! `rustc` (see `docs/implementation/driver.md`).
//!
//! Each `emit_diagnostic` runs one of two transforms, then a shared cleanup. A resolvable CGP
//! wiring failure is handed to the typed [`resolve`](crate::resolve)r and, when it succeeds, the
//! rustc-free [`plan_resolved`](cargo_cgp_error_processing::plan_resolved) words the replacement;
//! otherwise the wiring-message [`rewrite`](cargo_cgp_error_processing::rewrite) renames the notes
//! in place. Either way the diagnostic then passes through the
//! [`postprocess`](cargo_cgp_error_processing::postprocess) cleanup. The module is split into
//! [`install`] (rebuilding the compiler's default emitter and wrapping it), [`cgp_emitter`] (the
//! [`CgpEmitter`] type and its orchestration), and [`edit`] (the `DiagInner`-editing helpers).

mod cgp_emitter;
mod edit;
mod install;

pub use cgp_emitter::CgpEmitter;
pub use install::install;
