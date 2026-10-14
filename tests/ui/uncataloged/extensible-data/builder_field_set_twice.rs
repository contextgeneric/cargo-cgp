//! A `BuildWithHandlers` list that sets one field twice, asserted by a check.
//!
//! Each `BuildAndSetField` step builds its field into the partial record, which accepts a field only
//! while it is unset. The second step for `host` finds it set, so the check fails on the builder's
//! `UpdateField` bound with `Mapper == IsNothing`; the output never says `host` is already set.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E101 CGP-E102 CGP-E117

use core::marker::PhantomData;

use cgp::extra::dispatch::{BuildAndSetField, BuildWithHandlers};
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

pub struct DefaultHost;

#[cgp_provider]
impl<Context, Code, Builder> Computer<Context, Code, &Builder> for DefaultHost {
    type Output = String;

    fn compute(_context: &Context, _code: PhantomData<Code>, _builder: &Builder) -> String {
        "localhost".to_owned()
    }
}

pub struct DefaultPort;

#[cgp_provider]
impl<Context, Code, Builder> Computer<Context, Code, &Builder> for DefaultPort {
    type Output = u16;

    fn compute(_context: &Context, _code: PhantomData<Code>, _builder: &Builder) -> u16 {
        8080
    }
}

pub struct App;

delegate_components! {
    App {
        ComputerComponent: BuildWithHandlers<Config, Product![
            BuildAndSetField<Symbol!("host"), DefaultHost>,
            BuildAndSetField<Symbol!("port"), DefaultPort>,
            BuildAndSetField<Symbol!("host"), DefaultHost>,
        ]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), ()),
    }
}

fn main() {}
