//! Two `#[cgp_auto_dispatch]` traits in one module that both declare a method named `area`, each
//! attribute written by its path.
//!
//! The compiler records each expansion's macro as `cgp::prelude::cgp_auto_dispatch`, and the tool
//! reads a macro by its path's last segment, so the two `E0428`s pair into one `[CGP-E061]`, as for
//! the bare attribute (`auto_dispatch_shared_method_name`).
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md, with the name clash itself
//! under https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0428.md.

//@quality: acceptable
//@codes: CGP-E061

use cgp::prelude::*;

#[cgp::prelude::cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
}

#[cgp::prelude::cgp_auto_dispatch]
pub trait HasScaledArea {
    fn area(&self) -> f64;
}

fn main() {}
