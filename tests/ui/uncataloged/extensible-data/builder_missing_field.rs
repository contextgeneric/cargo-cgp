//! A `BuildWithHandlers` list without a step for one field.
//!
//! An uncoded `FinalizeBuild` headline over a tree; the unset `port` field is never named.
//!
//! CGP error class: none yet.
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

pub struct App;

delegate_components! {
    App {
        ComputerComponent: BuildWithHandlers<Config, Product![
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
