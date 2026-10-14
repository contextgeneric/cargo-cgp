//! A `BuildWithHandlers` list that sets one field twice, asserted by a check.
//!
//! Each `BuildAndSetField` step builds its field into the partial record, which accepts a field only
//! while it is unset. The second step for `host` finds it set, so the check fails on the builder's
//! `UpdateField` bound with `Mapper == IsNothing`, and the tool reads the markers:
//! ``[CGP-E001] field `host` of `Config` is already set``. The last step names the builder by the
//! fields it holds; the provider steps above it still show the builder's generated `__PartialConfig`
//! type as the pipeline's input.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E118

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
