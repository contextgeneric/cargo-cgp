//! The diagnostic-rewriting emitter the driver installs.
//!
//! This is the compiler-side seam of the message transform. It replaces the session's JSON
//! emitter with one that rewrites CGP wiring messages (via the rustc-free
//! [`rewrite`](cargo_cgp_error_processing::rewrite) module) before handing the diagnostic to
//! a real [`JsonEmitter`], so the transformed text reaches cargo — and the front-end —
//! already shaped, both in the structured `children` and in the `rendered` field the JSON
//! emitter regenerates from them.
//!
//! Two facts make this the right layer. First, naming the traits behind a component marker
//! needs the compiler, and the driver reaches the live `TyCtxt` through
//! [`rustc_middle::ty::tls`] because a wiring message is built during trait solving, when a
//! `TyCtxt` is in thread-local scope. That compiler lookup is wrapped as the `fn`-pointer
//! initializer of a [`ComponentNameMap`], which builds the map lazily on the first rewrite
//! and never at all for a diagnostic that mentions no CGP wiring — so the emitter needs no
//! separate "is this a CGP diagnostic?" pre-check. Second, mutating the `DiagInner` in place
//! before the inner emitter serializes it means both the JSON `children` and the regenerated
//! `rendered` text carry the rewrite, with no re-parsing of rendered output.
//!
//! The session's emitter cannot be *wrapped* — [`set_emitter`](rustc_errors::DiagCtxt::set_emitter)
//! only replaces it, with no way to recover the original — so the inner `JsonEmitter` is
//! rebuilt to match how the compiler builds its default one (see [`install`]).

use std::borrow::Cow;
use std::io;
use std::path::Path;

use cargo_cgp_error_processing::rewrite::{ComponentNameMap, rewrite_message};
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::json::JsonEmitter;
use rustc_errors::timings::TimingRecord;
use rustc_errors::{DiagInner, DiagMessage, TerminalUrl};
use rustc_interface::interface::Config;
use rustc_session::config::ErrorOutputType;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;

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

/// The wrapping [`Emitter`] that rewrites CGP wiring messages before delegating to the real
/// JSON emitter.
struct CgpEmitter {
    inner: JsonEmitter,
    /// The component-marker → trait-names map. A [`ComponentNameMap`] owns the laziness: its
    /// `fn`-pointer initializer ([`build_name_map_from_tls`]) runs the expensive
    /// whole-trait-graph walk at most once — on the first message that actually needs a
    /// lookup — and never when no diagnostic mentions CGP wiring, so this emitter needs no
    /// candidate pre-check of its own. Built once per compilation is sound because the map
    /// draws only on data fixed for the rest of the compilation (the trait set, the
    /// `IsProviderFor` supertraits, the blanket impls) and stores owned `String`s, not
    /// compiler handles.
    names: ComponentNameMap,
}

impl CgpEmitter {
    fn new(inner: JsonEmitter) -> Self {
        Self {
            inner,
            names: ComponentNameMap::new(build_name_map_from_tls),
        }
    }

    /// Rewrite every recognized CGP wiring message in `diag`, in place — both the primary
    /// header and the obligation-chain notes. A message that is not a wiring form is left
    /// untouched, and the name map is forced only when some message is actually rewritten.
    fn rewrite(&self, diag: &mut DiagInner) {
        rewrite_messages(&mut diag.messages, &self.names);
        for child in &mut diag.children {
            rewrite_messages(&mut child.messages, &self.names);
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

/// Rewrite each plain-string message in place, leaving its style and any Fluent message
/// untouched. Delegates the match-and-rewrite to [`rewrite_message`], which consults the name
/// map only for a message that parses as a CGP wiring form.
fn rewrite_messages<S>(messages: &mut [(DiagMessage, S)], names: &ComponentNameMap) {
    for (message, _) in messages.iter_mut() {
        if let DiagMessage::Str(text) = message
            && let Some(rewritten) = rewrite_message(text, names)
        {
            *message = DiagMessage::Str(Cow::Owned(rewritten));
        }
    }
}
