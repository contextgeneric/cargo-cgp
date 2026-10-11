//! A builder output whose field name does not match the merged target's.
//!
//! `Http { timeout_url }` is merged into an `App` that expects `url`. The output is byte-identical to
//! `build_and_merge_extra_field`: the same `CanBuildFrom<Http>` headline and tree, naming no field.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E101 CGP-E102

use core::marker::PhantomData;

use cgp::extra::dispatch::BuildAndMergeOutputs;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Db {
    pub db_path: String,
}

pub struct BuildDb;

#[cgp_provider]
impl<Context, Code, Input> Computer<Context, Code, Input> for BuildDb {
    type Output = Db;

    fn compute(_context: &Context, _code: PhantomData<Code>, _input: Input) -> Db {
        Db { db_path: "db".into() }
    }
}

pub struct BuildHttp;

#[cgp_provider]
impl<Context, Code, Input> Computer<Context, Code, Input> for BuildHttp {
    type Output = Http;

    fn compute(_context: &Context, _code: PhantomData<Code>, _input: Input) -> Http {
        Http::default()
    }
}

#[derive(CgpData)]
pub struct App {
    pub db_path: String,
    pub url: String,
}

pub struct AppBuilder;

delegate_components! {
    AppBuilder {
        ComputerComponent: BuildAndMergeOutputs<App, Product![BuildDb, BuildHttp]>,
    }
}

check_components! {
    AppBuilder {
        ComputerComponent: ((), ()),
    }
}

fn main() {}

#[derive(Default, CgpData)]
pub struct Http {
    pub timeout_url: String,
}
