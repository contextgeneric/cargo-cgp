//! A builder output with a field the merged target does not declare.
//!
//! `BuildAndMergeOutputs<App, Product![BuildDb, BuildHttp]>` merges `Http { url, timeout }` into an
//! `App` without `timeout`. The check fails with an uncoded `CanBuildFrom<Http>` headline over a
//! six-hop tree whose leaf repeats it; `timeout` is never named. The output is byte-identical to its
//! sibling `build_and_merge_missing_field`, a different mistake.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E101 CGP-E102 CGP-E117

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
