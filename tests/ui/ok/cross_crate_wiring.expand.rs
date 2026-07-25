#![feature(prelude_import)]
//! Positive cross-crate wiring: CGP's two-trait split stays within Rust's
//! coherence and orphan rules across crate boundaries. All the cross-crate wiring
//! lives in the auxiliary crates — `cgp-test-crate-b` (downstream) consumes
//! `cgp-test-crate-a` (upstream) to wire a foreign component to a foreign provider,
//! define a local provider for a foreign provider trait, join an upstream
//! namespace, and register a local component into an upstream namespace with
//! `#[default_impl]`. Building this fixture compiles both aux crates, so a clean
//! check confirms every cross-crate impl is coherent.
//!
//! This is the orphan-*safe* counterpart to the failing `wiring/orphan/` fixtures.
//!
//! CGP coherence concept:
//! <https://github.com/contextgeneric/cgp/blob/main/docs/concepts/coherence.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp_test_crate_a::{CanAnnounce, CanGreet};
use cgp_test_crate_b::{Broadcaster, CanFarewell, Leaver, Person};
fn main() {
    let person = Person { name: "John".to_owned() };
    let broadcaster = Broadcaster {
        name: "John".to_owned(),
    };
    let leaver = Leaver { name: "John".to_owned() };
    let _ = person.greet();
    let _ = broadcaster.announce();
    let _ = leaver.farewell();
}
