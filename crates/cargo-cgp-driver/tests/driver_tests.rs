//! The unit tests of the driver: rustc argument preparation, the expand marker flags, and the
//! help and version answers.
//!
//! One test target per crate, with a module per subject, keeps the crate to a single test binary.

// Linking `cargo-cgp-driver` links the compiler's `rustc_driver` dylib, which a crate may do only
// with the same `rustc_private` gate the driver binary carries.
#![feature(rustc_private)]

pub mod driver;
