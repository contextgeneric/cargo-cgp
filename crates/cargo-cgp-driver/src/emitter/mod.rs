//! The diagnostic-transforming emitter the driver installs.
//!
//! This is the compiler-side seam of every diagnostic transform. It replaces the session's own
//! emitter with [`CgpEmitter`], which acts on each diagnostic before handing it to a real inner
//! emitter, so the transformed result reaches cargo already shaped and rendered like vanilla
//! `rustc` (see `cgp-knowledge-base/cargo-cgp/implementation/driver.md`).
//!
//! Each `emit_diagnostic` runs one of two transforms, then a shared cleanup. A resolvable CGP
//! wiring failure is handed to the typed [`resolve`](crate::resolve)r and, when it succeeds, the
//! rustc-free [`plan_resolved`](cargo_cgp_error_processing::plan_resolved) words the replacement;
//! otherwise the wiring-message [`rewrite`](cargo_cgp_error_processing::rewrite) renames the notes
//! in place. Either way the diagnostic then passes through the
//! [`postprocess`](cargo_cgp_error_processing::postprocess) cleanup. A panic in that work, or in
//! the flush that renders the buffered diagnostics, falls back to the diagnostics rustc built.
//!
//! The module is split into [`install`] (rebuilding the compiler's default emitter and wrapping
//! it), [`cgp_emitter`] (the [`CgpEmitter`] type and its orchestration), [`edit`] (the
//! `DiagInner`-editing helpers), [`buffer`] (the diagnostics held until the flush), [`output`]
//! (the inner emitter and buffer the panic hook can reach), [`name_resolution`] (the pass that
//! reshapes a wiring entry's name-resolution errors once resolution is over), [`generated_names`] (the pass that drops a suggestion naming a generated identifier), [`containment`] (containing a panic
//! and the panic hooks), and [`test_panic`] (the panic a fixture raises on purpose).

mod buffer;
mod cgp_emitter;
mod containment;
mod edit;
mod generated_names;
mod install;
mod name_resolution;
mod output;
mod repeats;
mod shared_dispatch_method;
mod test_panic;

pub use cgp_emitter::CgpEmitter;
pub use containment::install_panic_hooks;
pub use generated_names::drop_generated_name_suggestions;
pub use install::install;
pub use name_resolution::reshape_name_resolution_errors;
pub use test_panic::*;
