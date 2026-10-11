//! A local wiring entry on an upstream context that joins a namespace.
//!
//! `Broadcaster`, from the `cgp-test-crate-b` auxiliary crate, joins `DefaultNamespace`, so that
//! crate has a blanket `DelegateComponent<__Key__>` impl forwarding every key the namespace
//! answers. This crate registers its own `LabelerComponent` into `DefaultNamespace` with
//! `#[default_impl]` and also wires it on `Broadcaster` directly, which the orphan rule allows,
//! since the key is local. The two now overlap (E0119). rustc names the upstream impl in a note
//! rather than at a labelled span, so the conflict classifier finds no local "first
//! implementation" to compare the entry with.
//!
//! Without the first entry the tool cannot say what the key collides with, so both `E0119`s keep
//! rustc's wording, each naming the upstream impl in its note. Calling the conflict a duplicate
//! entry would be false: the context has one entry for the key, and it collides with the namespace
//! the context joins upstream.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@aux-build: cgp-test-crate-a
//@aux-build: cgp-test-crate-b
//@quality: usability
//@codes: none

use cgp::prelude::*;
use cgp_test_crate_b::Broadcaster;

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

delegate_components! {
    Broadcaster {
        LabelerComponent: FixedLabel,
    }
}

fn main() {}
