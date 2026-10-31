//! The blocks of text the run prints: a fixture's summary, its raw output, and the fixture list.

use std::path::{Path, PathBuf};

use crate::snapshot::Outcome;
use crate::{directives, harness};

/// Render one fixture's summary line, plus any mismatch diffs beneath it. Returns the
/// block ending in a newline so the caller can print blocks back to back.
pub fn report_block(name: &str, outcomes: &[(&str, Outcome)]) -> String {
    let mut diffs = String::new();
    let mut mismatched: Vec<&str> = Vec::new();
    let mut blessed = false;

    for (label, outcome) in outcomes {
        match outcome {
            Outcome::Mismatch(diff) => {
                mismatched.push(label);
                diffs.push_str(diff);
            }
            Outcome::Blessed => blessed = true,
            Outcome::Ok => {}
        }
    }

    if !mismatched.is_empty() {
        format!("MISMATCH {name}  ({})\n{diffs}", mismatched.join(", "))
    } else if blessed {
        format!("blessed  {name}\n")
    } else {
        format!("ok       {name}\n")
    }
}

/// Render a fixture's raw output for interactive inspection: the tool's stderr, plain
/// `cargo check`'s stderr, and the expansion, each unnormalized and labelled with its exit status.
pub fn print_block(
    harness_crate: &Path,
    fixture: &Path,
    name: &str,
    env: &[(String, String)],
) -> String {
    let status = |ok: bool| if ok { "success" } else { "failure" };
    let cgp = harness::run_fixture(harness_crate, fixture, env);
    let rust = harness::run_fixture_rust(harness_crate, fixture);
    let expand = harness::run_fixture_expand(harness_crate, fixture, env);
    format!(
        "===== {name}: cargo cgp check ({}) =====\n{}\
         ===== {name}: cargo check ({}) =====\n{}\
         ===== {name}: cargo cgp expand =====\n{}\
         ===== end {name} =====\n",
        status(cgp.success),
        cgp.stderr,
        status(rust.success),
        rust.stderr,
        expand,
    )
}

/// List the selected fixtures with their declared quality and codes, one per line, without
/// compiling anything. The quality leads each line, so `--list | grep ^usability` finds every
/// fixture whose presentation still needs work.
pub fn list_block(fixtures_dir: &Path, fixtures: &[PathBuf]) -> String {
    let mut out = String::new();
    for fixture in fixtures {
        let name = fixture
            .strip_prefix(fixtures_dir)
            .unwrap_or(fixture)
            .display();
        let directives = directives::read(fixture);
        let quality = directives.quality.as_deref().unwrap_or("-");
        let codes = match &directives.codes {
            Some(codes) if codes.is_empty() => "none".to_owned(),
            Some(codes) => codes.join(" "),
            None => "-".to_owned(),
        };
        out.push_str(&format!("{quality:<11} {name}  [{codes}]\n"));
    }
    out
}
