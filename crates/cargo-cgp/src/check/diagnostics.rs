//! Capturing cargo's JSON diagnostics and re-emitting the processed result.
//!
//! The front-end runs `cargo check --message-format=json`, so cargo's diagnostics arrive
//! as a JSON stream on stdout instead of pretty text on stderr. This module parses that
//! stream into the [`Diagnostic`] values the processing stage consumes, and writes the
//! processed result back out by printing each diagnostic's rustc-`rendered` text — which
//! reproduces rustc's own pretty output.

use std::collections::HashSet;
use std::io::{self, Write};

use cargo_cgp_error_processing::CgpDiagnostic;
use cargo_metadata::Message;
use cargo_metadata::diagnostic::Diagnostic;

/// The parsed output of one `cargo check --message-format=json` run: the compiler
/// diagnostics, and any non-JSON stdout lines to forward verbatim.
pub struct CapturedOutput {
    /// The compiler diagnostics, in the order cargo emitted them.
    pub diagnostics: Vec<Diagnostic>,
    /// Stray non-JSON lines cargo wrote to stdout, preserved so nothing is dropped.
    pub text_lines: Vec<String>,
}

/// Parse cargo's JSON message stream from captured stdout, separating compiler
/// diagnostics from any stray non-JSON stdout lines. Artifact, build-script, and
/// build-finished messages carry no diagnostic and are dropped; a malformed line is not
/// a diagnostic and is ignored.
pub fn parse_cargo_output(stdout: &[u8]) -> CapturedOutput {
    let mut diagnostics = Vec::new();
    let mut text_lines = Vec::new();

    for message in Message::parse_stream(stdout) {
        match message {
            Ok(Message::CompilerMessage(compiler_message)) => {
                diagnostics.push(compiler_message.message);
            }
            Ok(Message::TextLine(line)) => text_lines.push(line),
            Ok(_) | Err(_) => {}
        }
    }

    CapturedOutput {
        diagnostics,
        text_lines,
    }
}

/// Write the processed diagnostics to `out`, emitting each one's rustc-rendered text so
/// the result matches what rustc itself would have printed. A diagnostic with no rendered
/// form (rare) contributes nothing.
///
/// Exact-duplicate renderings are suppressed, reproducing a step rustc's *human* emitter
/// performs but its JSON emitter does not: when the same diagnostic is produced more than
/// once, the terminal shows it a single time (the error *count* still includes the
/// repeats). Capturing via `--message-format=json` loses that suppression, so restoring
/// it here keeps the output identical to rustc's own. This is rendering fidelity, not CGP
/// analysis — the cascade-collapsing deduplication belongs to `process_cgp_errors`.
pub fn emit_rendered(out: &mut impl Write, processed: &[CgpDiagnostic]) -> io::Result<()> {
    let mut seen = HashSet::new();
    for diagnostic in processed {
        if let Some(rendered) = diagnostic.rendered()
            && seen.insert(rendered)
        {
            out.write_all(rendered.as_bytes())?;
        }
    }
    Ok(())
}
