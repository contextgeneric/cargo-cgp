//! A `BuildWithHandlers` list without a step for one field.
//!
//! The check reads the builder's presence markers when its `FinalizeBuild` bound fails:
//! ``[CGP-E001] field `port` of `Config` is not set``, under the chain through `BuildWithHandlers`.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E119

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
