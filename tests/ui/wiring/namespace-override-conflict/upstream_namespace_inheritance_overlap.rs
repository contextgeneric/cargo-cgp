//! A local key registered into an upstream namespace and into the parent it inherits.
//!
//! `AppNamespace`, from the `cgp-test-crate-a` auxiliary crate, inherits `DefaultNamespace`, so
//! that crate has a blanket impl of `AppNamespace` forwarding every key the parent answers. This
//! crate registers its own `LabelerComponent` into both, which the orphan rule allows, since the key
//! is local. The registration in `AppNamespace` then overlaps the upstream inheritance blanket
//! (E0119). rustc names the upstream impl in a note rather than at a labelled span, so the conflict
//! classifier finds no local "first implementation" to compare the entry with, and the diagnostic
//! keeps rustc's wording.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md.
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@aux-build: cgp-test-crate-a
//@quality: usability
//@codes: none

use cgp::prelude::*;
use cgp_test_crate_a::AppNamespace;

#[cgp_component(Labeler)]
pub trait CanLabel {
    fn label(&self) -> String;
}

#[cgp_impl(new FixedLabel)]
#[default_impl(LabelerComponent in DefaultNamespace)]
impl Labeler {
    fn label(&self) -> String {
        "label".to_owned()
    }
}

#[cgp_impl(new OtherLabel)]
#[default_impl(LabelerComponent in AppNamespace)]
impl Labeler {
    fn label(&self) -> String {
        "other".to_owned()
    }
}

fn main() {}
