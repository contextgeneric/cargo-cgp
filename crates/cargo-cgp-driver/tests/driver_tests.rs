//! The unit tests of the driver: rustc argument preparation, the expand marker flags, the help
//! and version answers, the panic containment, the emitter's stage pipeline and its apply step,
//! the overflow drop for an unresolved wiring value, the post-processing of span labels, the merge
//! of a diagnostic rustc reports twice, the rebuilt explanation footer, and the anchor list's gates.
//!
//! One test target per crate, with a module per subject, keeps the crate to a single test binary.

// Linking `cargo-cgp-driver` links the compiler's `rustc_driver` dylib, which a crate may do only
// with the same `rustc_private` gate the driver binary carries.
#![feature(rustc_private)]

extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_span;

pub mod driver;
