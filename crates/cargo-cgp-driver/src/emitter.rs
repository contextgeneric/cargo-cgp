//! The diagnostic-rewriting emitter the driver installs.
//!
//! This is the compiler-side seam of the message transform. It replaces the session's JSON
//! emitter with one that rewrites CGP wiring notes (via [`crate::rewrite`]) before handing
//! the diagnostic to a real [`JsonEmitter`], so the transformed text reaches cargo — and
//! the front-end — already shaped, both in the structured `children` and in the `rendered`
//! field the JSON emitter regenerates from them.
//!
//! Two facts make this the right layer. First, naming the traits behind a component marker
//! needs the compiler ([`build_component_name_map`]), and the emitter can reach the live
//! `TyCtxt` through [`rustc_middle::ty::tls`] because a wiring note is built during trait
//! solving, when a `TyCtxt` is in thread-local scope. Second, mutating the `DiagInner` in
//! place before the inner emitter serializes it means both the JSON `children` and the
//! regenerated `rendered` text carry the rewrite, with no re-parsing of rendered output.
//!
//! The session's emitter cannot be *wrapped* — [`set_emitter`](rustc_errors::DiagCtxt::set_emitter)
//! only replaces it, with no way to recover the original — so the inner `JsonEmitter` is
//! rebuilt to match how the compiler builds its default one (see [`install`]).

use std::borrow::Cow;
use std::collections::HashMap;
use std::io;
use std::path::Path;

use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::json::JsonEmitter;
use rustc_errors::timings::TimingRecord;
use rustc_errors::{DiagInner, DiagMessage, TerminalUrl};
use rustc_interface::interface::Config;
use rustc_middle::ty;
use rustc_session::config::ErrorOutputType;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_component_name_map;
use crate::rewrite::{ComponentTraitNames, is_wiring_note, rewrite_required_for};

/// Install the rewriting emitter on the compiler session, replicating how rustc builds its
/// default JSON emitter so cargo's diagnostic stream is byte-for-byte the same apart from
/// the rewritten CGP notes.
///
/// It only acts on the JSON error format — the one the front-end drives cargo with, and the
/// only one whose output the tool consumes; a human-format invocation (e.g. running the
/// driver by hand) is left with the compiler's own emitter. The session options the emitter
/// construction needs are not reachable from `psess_created`'s `&mut ParseSess`, so they are
/// read here from [`Config::opts`] and moved into the callback.
pub fn install(config: &mut Config) {
    let sopts = &config.opts;

    let (pretty, json_rendered, color_config) = match &sopts.error_format {
        ErrorOutputType::Json {
            pretty,
            json_rendered,
            color_config,
        } => (*pretty, *json_rendered, *color_config),
        ErrorOutputType::HumanReadable { .. } => return,
    };

    let macro_backtrace = sopts.unstable_opts.macro_backtrace;
    let track_diagnostics = sopts.unstable_opts.track_diagnostics;
    let ui_testing = sopts.unstable_opts.ui_testing;
    let link_only = sopts.unstable_opts.link_only;
    let diagnostic_width = sopts.diagnostic_width;
    let ignored_directories = sopts
        .unstable_opts
        .ignore_directory_in_diagnostics_source_blocks
        .clone();
    let terminal_url = resolve_terminal_url(
        sopts.unstable_opts.terminal_urls,
        sopts.unstable_features.is_nightly_build(),
    );

    config.psess_created = Some(Box::new(move |psess| {
        let source_map = (!link_only).then(|| psess.clone_source_map());
        let inner = JsonEmitter::new(
            Box::new(io::BufWriter::new(io::stderr())),
            source_map,
            pretty,
            json_rendered,
            color_config,
        )
        .ui_testing(ui_testing)
        .ignored_directories_in_source_blocks(ignored_directories)
        .diagnostic_width(diagnostic_width)
        .macro_backtrace(macro_backtrace)
        .track_diagnostics(track_diagnostics)
        .terminal_url(terminal_url);

        psess.dcx().set_emitter(Box::new(CgpEmitter::new(inner)));
    }));
}

/// Resolve `--terminal-urls=auto` the same way `rustc_session::session::default_emitter`
/// does, so the rebuilt emitter matches the compiler's default rather than guessing.
fn resolve_terminal_url(setting: TerminalUrl, is_nightly: bool) -> TerminalUrl {
    match setting {
        TerminalUrl::Auto => {
            match (
                std::env::var("COLORTERM").as_deref(),
                std::env::var("TERM").as_deref(),
            ) {
                (Ok("truecolor"), Ok("xterm-256color")) if is_nightly => TerminalUrl::Yes,
                _ => TerminalUrl::No,
            }
        }
        other => other,
    }
}

/// The wrapping [`Emitter`] that rewrites CGP wiring notes before delegating to the real
/// JSON emitter. It caches the component-name map so the whole-crate compiler queries that
/// build it run at most once, and only when a diagnostic actually carries a wiring note.
struct CgpEmitter {
    inner: JsonEmitter,
    names: Option<HashMap<String, ComponentTraitNames>>,
}

impl CgpEmitter {
    fn new(inner: JsonEmitter) -> Self {
        Self { inner, names: None }
    }

    /// Rewrite every recognized wiring note in `diag`, in place. No-op unless the
    /// diagnostic carries a candidate note *and* a `TyCtxt` is reachable to name the traits.
    fn rewrite(&mut self, diag: &mut DiagInner) {
        if !diag_has_wiring_note(diag) {
            return;
        }
        if self.names.is_none() {
            // A wiring note is emitted during trait solving, so a `TyCtxt` is in TLS; if it
            // is somehow absent, leave the diagnostic untouched and retry on the next one.
            match ty::tls::with_opt(|tcx| tcx.map(build_component_name_map)) {
                Some(map) => self.names = Some(map),
                None => return,
            }
        }
        let names = self.names.as_ref().expect("name map built above");
        rewrite_messages(&mut diag.messages, names);
        for child in &mut diag.children {
            rewrite_messages(&mut child.messages, names);
        }
    }
}

impl Emitter for CgpEmitter {
    fn emit_diagnostic(&mut self, mut diag: DiagInner) {
        self.rewrite(&mut diag);
        self.inner.emit_diagnostic(diag);
    }

    fn source_map(&self) -> Option<&SourceMap> {
        self.inner.source_map()
    }

    fn emit_artifact_notification(&mut self, path: &Path, artifact_type: &str) {
        self.inner.emit_artifact_notification(path, artifact_type);
    }

    fn emit_timing_section(&mut self, record: TimingRecord, event: TimingEvent) {
        self.inner.emit_timing_section(record, event);
    }

    fn emit_future_breakage_report(&mut self, diags: Vec<DiagInner>) {
        self.inner.emit_future_breakage_report(diags);
    }

    fn emit_unused_externs(&mut self, lint_level: rustc_lint_defs::Level, unused_externs: &[&str]) {
        self.inner.emit_unused_externs(lint_level, unused_externs);
    }

    fn should_show_explain(&self) -> bool {
        self.inner.should_show_explain()
    }
}

/// Whether any message in the diagnostic looks like a rewritable wiring note, used to skip
/// the map-building queries for the common non-CGP diagnostic.
fn diag_has_wiring_note(diag: &DiagInner) -> bool {
    messages_have_wiring_note(&diag.messages)
        || diag
            .children
            .iter()
            .any(|child| messages_have_wiring_note(&child.messages))
}

fn messages_have_wiring_note<S>(messages: &[(DiagMessage, S)]) -> bool {
    messages
        .iter()
        .any(|(message, _)| matches!(message, DiagMessage::Str(s) if is_wiring_note(s)))
}

/// Rewrite each plain-string message in place, leaving its style and any Fluent message
/// untouched.
fn rewrite_messages<S>(
    messages: &mut [(DiagMessage, S)],
    names: &HashMap<String, ComponentTraitNames>,
) {
    for (message, _) in messages.iter_mut() {
        if let DiagMessage::Str(text) = message
            && let Some(rewritten) = rewrite_required_for(text, names)
        {
            *message = DiagMessage::Str(Cow::Owned(rewritten));
        }
    }
}
