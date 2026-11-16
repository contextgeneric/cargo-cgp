//! The diagnostic-transforming emitter the driver installs.
//!
//! This is the compiler-side seam of every diagnostic transform. It replaces the session's own
//! emitter with [`DeferringEmitter`], whose [`CgpEmitter`] acts on each diagnostic before handing
//! it to a real inner emitter, so the transformed result reaches cargo already shaped and rendered
//! like vanilla `rustc` (see `cgp-knowledge-base/cargo-cgp/implementation/driver/program.md`).
//!
//! Each `emit_diagnostic` first tries the reshapes of specific compiler errors (a wiring conflict,
//! a structural mistake, an undeclared dependency, and the like). Any other diagnostic runs one of
//! two transforms, then a shared cleanup. A resolvable CGP wiring failure is handed to the typed
//! [`resolve`](crate::resolve)r and, when it succeeds, the rustc-free
//! [`plan_resolved`](cargo_cgp_error_processing::plan_resolved) words the replacement; otherwise
//! the wiring-message [`rewrite`](cargo_cgp_error_processing::rewrite) renames the notes in place.
//! Either way the diagnostic then passes through the
//! [`postprocess`](cargo_cgp_error_processing::postprocess) cleanup. A panic in that work, or in
//! the flush that renders the buffered diagnostics, falls back to the diagnostics rustc built.
//!
//! The module is split into `install` (rebuilding the compiler's default emitter and wrapping
//! it), `cgp_emitter` (the [`CgpEmitter`] type and its orchestration, and the [`DeferringEmitter`]
//! the session holds), `deferred` (the queue of diagnostics emitted during rustc's `analysis`
//! query, transformed once it ends), `pipeline` (the verdict a
//! stage returns and the step that writes a reshape), `stage` (the stages, one module each),
//! `edit` (the `DiagInner`-editing helpers), `buffer` (the diagnostics held until the flush),
//! `output` (the inner emitter and buffer the panic hook can reach), `name_resolution` (the pass
//! that reshapes a wiring entry's name-resolution errors once resolution is over),
//! `generated_names` (the pass that drops a suggestion naming a generated identifier),
//! `use_type_misspelling` (the pass that names the rename a misspelled `#[use_type]` import
//! needs), `attribute_splice` (the drop of a suggestion spliced after an attribute macro),
//! `repeats` (the flush's merge of a diagnostic rustc reports twice), `order` (the canonical order
//! the queue and the flush follow), `containment` (containing a panic and the panic hooks), and
//! `fallback` (the diagnostics emitted when a transform or the flush panics).

mod attribute_splice;
mod buffer;
mod buffer_pass;
mod cgp_emitter;
mod containment;
mod crate_facts;
mod deferred;
mod edit;
mod fallback;
mod flush;
mod generated_names;
mod install;
mod name_resolution;
mod order;
mod output;
mod pipeline;
mod repeats;
mod stage;
mod use_type_misspelling;

pub use attribute_splice::drop_attribute_splices;
pub use buffer::*;
pub use cgp_emitter::{CgpEmitter, DeferringEmitter};
pub use containment::{contain, install_panic_hooks, is_closed_stdout_panic};
pub use deferred::{Handling, MAX_ROUNDS, drain_queue, override_analysis};
pub use edit::{merged_span, postprocess_multispan};
pub use fallback::*;
pub use flush::*;
pub use generated_names::{drop_generated_name_suggestions, drop_generated_names};
pub use install::install;
pub use name_resolution::{is_unresolved_wiring_overflow, reshape_name_resolution_errors};
pub use order::{order_key, sort_canonically};
pub use output::*;
pub use pipeline::*;
pub use repeats::merge_repeats;
pub use stage::shared_dispatch_method;
pub use use_type_misspelling::reshape_use_type_misspellings;
