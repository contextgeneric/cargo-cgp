//! Span-independent de-duplication of the tool's transformed diagnostics.
//!
//! CGP wiring is lazy, so one mistake surfaces the same error at many sites: the
//! `check_components!` entry, every hand-written `impl` that references the broken component, and
//! each call. The emitter records each transformed diagnostic here and suppresses a later one whose
//! signature it has already seen, so a mistake is shown once. Every key is span-independent, since
//! the copies differ in exactly the span. Only the tool's own transformed diagnostics
//! are recorded; an untouched `rustc` error never reaches this ledger.

use std::collections::HashSet;

/// The ledger of CGP diagnostics already emitted this compilation, keyed by span-independent
/// signature.
#[derive(Debug, Default)]
pub struct DedupLedger {
    seen: HashSet<String>,
}

impl DedupLedger {
    /// A ledger with nothing recorded.
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether a resolved diagnostic recorded the coded class headline `header`, so a declined
    /// report carrying it restates that resolved one (see
    /// [`check_and_record`](Self::check_and_record)).
    pub fn records_header(&self, header: &str) -> bool {
        self.seen.contains(&format!("header\u{1f}{header}"))
    }

    /// Record one transformed diagnostic and report whether an equal one was already recorded:
    /// `true` means this diagnostic is a re-report the caller should suppress.
    ///
    /// See `cgp-knowledge-base/cargo-cgp/implementation/driver/program.md` for the three keys and
    /// why the coded headline works in one direction only.
    pub fn check_and_record(
        &mut self,
        cause_signature: Option<&str>,
        text_signature: impl FnOnce() -> String,
        headers: &[&str],
    ) -> bool {
        let headers: Vec<String> = headers
            .iter()
            .filter(|header| header.starts_with("[CGP-E0"))
            .map(|header| format!("header\u{1f}{header}"))
            .collect();
        match cause_signature {
            Some(cause) => {
                self.seen.extend(headers);
                !self.seen.insert(format!("cause\u{1f}{cause}"))
            }
            None => {
                let declined_header_seen = headers.iter().any(|header| self.seen.contains(header));
                !self.seen.insert(format!("text\u{1f}{}", text_signature())) || declined_header_seen
            }
        }
    }
}
