//! Building with defaults when a field the source lacks has no `Default`.
//!
//! `Server::build_with_default(Host { … })` fails on `CanFinalizeWithDefault` for the builder with
//! `host` set. The data-call anchor reads the type-relative call and walks the defaulted finalize
//! to the bound it needs for the unset `port`:
//! ``[CGP-E050] the trait bound `Port: Default` is not satisfied``. A step of the chain names the
//! internal `TransformMapDefault` transform.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E050 CGP-E104 CGP-E117

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
