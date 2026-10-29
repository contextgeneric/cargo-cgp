//! Well-known names shared with the front-end and with CGP's crates.
//!
//! The names are compile-time constants passed into the functions that use them: the contract
//! with the front-end and the crash-report address (`front_end`), the compiler flags the driver
//! injects (`rustc_flags`), and the CGP items the resolver and the detectors anchor to by `DefId`,
//! grouped by the crate that defines them, with the names of CGP's macros and of what their
//! expansions generate (`cgp_macros`).

mod cgp_base_types;
mod cgp_component;
mod cgp_crates;
mod cgp_field;
mod cgp_field_extra;
mod cgp_macros;
mod cgp_type;
mod front_end;
mod rustc_flags;

pub use cgp_base_types::*;
pub use cgp_component::*;
pub use cgp_crates::*;
pub use cgp_field::*;
pub use cgp_field_extra::*;
pub use cgp_macros::*;
pub use cgp_type::*;
pub use front_end::*;
pub use rustc_flags::*;
