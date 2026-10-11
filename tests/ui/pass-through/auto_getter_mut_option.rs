//! A `&mut self` auto getter returning `Option<&mut String>`, a known `cgp` defect.
//!
//! rustc's suggestion is spliced after the attribute, and the tool prints it with full paths
//! (`std::option::Option<&mut std::string::String>`).
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@json
//@rewrites-falsely: untransformed output prints full paths, because the driver does not set `trimmed_def_paths`

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasNickname {
    fn nickname(&mut self) -> Option<&String>;
}

fn main() {}
