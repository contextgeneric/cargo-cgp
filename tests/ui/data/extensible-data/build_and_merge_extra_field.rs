//! A builder output with a field the merged target does not declare.
//!
//! `BuildAndMergeOutputs<App, Product![BuildDb, BuildHttp]>` merges `Http { url, timeout }` into an
//! `App` without `timeout`. The walk follows the merge into the builder's `BuildField` for each field
//! and reads the one `App` lacks: ``[CGP-E001] missing field `timeout` on `App` ``. No help adds the
//! field, since a merge can still leave another field unset (see `build_and_merge_missing_field`).
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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
    pub url: String,
    pub timeout: u64,
}
