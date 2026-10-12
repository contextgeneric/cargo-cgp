//! Building with defaults when a field the source lacks has no `Default`.
//!
//! `Server::build_with_default(Host { … })` fails on `CanFinalizeWithDefault` for
//! `__PartialServer<IsPresent, IsNothing>`. The output never names `port` or the missing
//! `Port: Default`, which the optional-field reference quotes as the error.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/hidden-root-cause.md.

//@quality: hidden
//@codes: none

use cgp::extra::field::impls::CanBuildWithDefault;
use cgp::prelude::*;

pub struct Port(pub u16);

#[derive(CgpData)]
pub struct Host {
    pub host: String,
}

#[derive(CgpData)]
pub struct Server {
    pub host: String,
    pub port: Port,
}

fn main() {
    let _ = Server::build_with_default(Host { host: "h".into() });
}
