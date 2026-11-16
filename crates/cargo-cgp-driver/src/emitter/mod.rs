//! The diagnostic-transforming emitter the driver installs: [`DeferringEmitter`], whose
//! [`CgpEmitter`] acts on each diagnostic before handing it to a real inner emitter, so the result
//! reaches cargo shaped and rendered like vanilla `rustc`.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/driver/program.md` and
//! `cgp-knowledge-base/cargo-cgp/implementation/driver/diagnostic-reshapes.md`.

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
mod registry;
mod repeats;
mod stage;
mod use_type_misspelling;

pub use attribute_splice::drop_attribute_splices;
pub use buffer::*;
pub use cgp_emitter::{CgpEmitter, DeferringEmitter};
pub use containment::{contain, install_panic_hooks, is_closed_stdout_panic};
pub use deferred::{
    Handling, KeptFatal, MAX_ROUNDS, contained_lookup, drain_queue, override_analysis, unless_fatal,
};
pub use edit::{merged_span, postprocess_multispan};
pub use fallback::*;
pub use flush::*;
pub use generated_names::{drop_generated_name_suggestions, drop_generated_names};
pub use install::install;
pub use name_resolution::{is_unresolved_wiring_overflow, reshape_name_resolution_errors};
pub use order::{order_key, sort_canonically};
pub use output::*;
pub use pipeline::*;
pub use registry::{LocalSlot, Registry};
pub use repeats::merge_repeats;
pub use stage::shared_dispatch_method;
pub use use_type_misspelling::reshape_use_type_misspellings;
