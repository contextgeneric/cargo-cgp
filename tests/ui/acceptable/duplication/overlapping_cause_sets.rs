//! Failures whose root-cause *sets* overlap without being equal, coalesced into one block.
//!
//! One omitted wiring entry — a single generic key covering every element type — is reached at two
//! instantiations, so it surfaces as two distinct missing-delegate root causes. Each
//! `check_components!` entry reaches one of the two; the use-site call walks every wired component
//! and so reaches both. No two of those three cause sets are equal, so grouping failures by an
//! *identical* cause set left one mistake reported as three blocks — and the use-site block fared
//! worst, because its two top-level roots were exactly the consumers the check blocks had already
//! drawn, so its chain was fully elided against them and it degenerated to a bare `root causes:`
//! list with no dependency chain at all, on the one diagnostic sitting on code the programmer wrote.
//!
//! Pins the shared-cause grouping that replaced it: the three failures form one group because each
//! shares a cause with the union block, so a single `[CGP-E001]` block names both affected consumers,
//! carets all three sites, lists both root causes, and renders both chains in full.
//!
//! Reproduces the [check-trait failure] class.
//!
//! [check-trait failure]:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md

use core::fmt::Display;

use cgp::prelude::*;

#[cgp_component(Encoder)]
pub trait CanEncode<Value> {
    fn encode(&self, value: &Value) -> String;
}

/// Encodes any `Display` value directly.
#[cgp_impl(new EncodeDisplay)]
impl<Value: Display> Encoder<Value> {
    fn encode(&self, value: &Value) -> String {
        value.to_string()
    }
}

/// The wrapper a container encoder hands each of its elements to. One generic wiring entry covers
/// every element type at once, so omitting that entry leaves one missing key per element type the
/// wiring actually reaches.
pub struct Element<Value>(pub Value);

#[cgp_component(FirstReporter)]
pub trait CanReportFirst {
    fn report_first(&self) -> String;
}

#[cgp_impl(new ReportFirst)]
#[uses(CanEncode<Element<u32>>)]
impl FirstReporter {
    fn report_first(&self) -> String {
        self.encode(&Element(1u32))
    }
}

#[cgp_component(SecondReporter)]
pub trait CanReportSecond {
    fn report_second(&self) -> String;
}

#[cgp_impl(new ReportSecond)]
#[uses(CanEncode<Element<u64>>)]
impl SecondReporter {
    fn report_second(&self) -> String {
        self.encode(&Element(2u64))
    }
}

pub struct App;

delegate_components! {
    App {
        open EncoderComponent;

        FirstReporterComponent:
            ReportFirst,
        SecondReporterComponent:
            ReportSecond,

        // The one entry covering every element type is missing, which is the whole mistake:
        //
        //     @EncoderComponent.<Value> Element<Value>: EncodeElement,

        @EncoderComponent.[
            u32,
            u64,
        ]:
            EncodeDisplay,
    }
}

check_components! {
    App {
        FirstReporterComponent,
        SecondReporterComponent,
    }
}

fn main() {
    let app = App;

    let _ = app.report_first();
}
