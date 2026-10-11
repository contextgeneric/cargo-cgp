//! `compute` called by bare name where two traits provide it.
//!
//! rustc's `E0034` suggestions print `cgp::cgp_extra::cgp_handler::CanCompute` under the tool and
//! `<App as cgp::prelude::Computer>` without its generics.
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

use cgp::extra::handler::CanCompute;

delegate_components! {
    App {
        ComputerComponent: Double,
    }
}

fn main() {
    let _ = App::compute(&App, PhantomData::<()>, 21);
}
