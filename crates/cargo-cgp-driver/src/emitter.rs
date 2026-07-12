//! The diagnostic-rewriting emitter the driver installs.
//!
//! This is the compiler-side seam of the message transform. It replaces the session's JSON
//! emitter with one that acts on each diagnostic before handing it to a real [`JsonEmitter`],
//! so the transformed result reaches cargo — and the front-end — already shaped, both in the
//! structured `children` and in the `rendered` field the JSON emitter regenerates from them.
//! It transforms a resolvable CGP wiring failure — a diagnostic on a `check_components!` entry, or
//! a broken consumer-method call (`E0599`) at its use site — into its root-cause dependency tree(s),
//! recovered from the compiler by [`resolve`](crate::resolve). A failure that bottoms out entirely on
//! missing *fields* is **replaced wholesale** with a fresh, tree-first diagnostic (custom header
//! naming the field(s), a derive `help`, one tree note each). A failure that bottoms out on any other
//! bound **keeps rustc's own main message** and only swaps its sub-notes (and structured suggestions)
//! for the tree. Everything the resolver cannot handle falls back to a **text rewrite** of the
//! compiler's own diagnostic, renaming CGP wiring messages via the rustc-free
//! [`rewrite`](cargo_cgp_error_processing::rewrite) module.
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

use cargo_cgp_error_processing::render_dependency_tree;
use cargo_cgp_error_processing::rewrite::{ComponentNameMap, rewrite_message};
use rustc_errors::codes::{E0277, E0599};
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::json::JsonEmitter;
use rustc_errors::timings::TimingRecord;
use rustc_errors::{DiagInner, DiagMessage, Level, MultiSpan, Style, Subdiag, TerminalUrl};
use rustc_interface::interface::Config;
use rustc_session::config::ErrorOutputType;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;
use crate::resolve::{self, Cause, FieldIssue, Leaf, Resolved};

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

    /// Resolve `diag`'s CGP wiring failure to its root-cause dependency tree(s), or `None` when
    /// this is not a resolvable wiring diagnostic (so the caller falls back to the in-place text
    /// rewrite). A candidate is any diagnostic whose messages mention a CGP wiring trait and whose
    /// caret sits on a `check_components!` entry; [`resolve::resolve_check_failure`] does the typed
    /// work and yields `None` for everything it cannot fully resolve. Returns the primary span
    /// alongside the resolution so the field-replacement path can re-aim the caret at the entry.
    fn try_resolve(&self, diag: &DiagInner) -> Option<(Resolved, Span)> {
        // Attempt resolution for any diagnostic that names a wiring trait, and for every method
        // `E0599` — a broken consumer call whose text may name only the user's own traits. The
        // use-site resolver declines cheaply when the caret is not on a CGP context.
        if !mentions_wiring(diag) && diag.code != Some(E0599) {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        let resolved = rustc_middle::ty::tls::with_opt(|tcx| {
            let tcx = tcx?;
            // Prefer the check-entry anchor (an obligation recovered from the check impl at the
            // caret). Failing that — a use-site failure such as a consumer-method call, whose
            // obligation no check impl carries — recover the context from the diagnostic's spans.
            resolve::resolve_check_failure(tcx, primary_span, &self.names)
                .or_else(|| resolve::resolve_use_site(tcx, &diagnostic_spans(diag), &self.names))
        })?;
        Some((resolved, primary_span))
    }
}

/// Every span a diagnostic carries — its primary and labelled spans plus each child's — the pool
/// the use-site resolver searches for one that lands on the failing context's type definition.
fn diagnostic_spans(diag: &DiagInner) -> Vec<Span> {
    let mut spans: Vec<Span> = diag.span.primary_spans().to_vec();
    spans.extend(diag.span.span_labels().into_iter().map(|label| label.span));
    for child in &diag.children {
        spans.extend(child.span.primary_spans());
        spans.extend(child.span.span_labels().into_iter().map(|label| label.span));
    }
    spans
}

/// Whether any of `diag`'s messages — its header or a child's — mentions a CGP wiring trait. This
/// is the cheap pre-filter that decides whether to attempt the (expensive) typed resolution at all,
/// standing in for the old `E0277`-only gate so that any wiring diagnostic is considered.
fn mentions_wiring(diag: &DiagInner) -> bool {
    fn any(messages: &[(DiagMessage, Style)]) -> bool {
        messages.iter().any(|(message, _)| match message {
            // `HasField` catches a use-site failure (a consumer-method `E0599`), whose text names
            // the missing leaf but not `CanUseComponent`/`IsProviderFor`.
            DiagMessage::Str(text) => {
                text.contains("CanUseComponent")
                    || text.contains("IsProviderFor")
                    || text.contains("HasField")
            }
            _ => false,
        })
    }
    any(&diag.messages) || diag.children.iter().any(|child| any(&child.messages))
}

/// A `and`-joined, back-quoted list of field names: `\`x\``, `\`x\` and \`y\``, or
/// `\`x\`, \`y\`, and \`z\``.
fn quoted_field_list(fields: &[String]) -> String {
    let quoted: Vec<String> = fields.iter().map(|f| format!("`{f}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The header for an all-field resolution. When every field is genuinely absent the header reads as
/// a missing field; when at least one field the context carries is unwired it reads as `HasField`
/// being unimplemented, since "missing" would misdescribe a field the struct visibly carries — the
/// fix (adding the derive) is carried by a separate `help` subdiagnostic.
fn field_header(resolved: &Resolved) -> String {
    let fields: Vec<String> = resolved
        .causes
        .iter()
        .filter_map(|cause| match &cause.leaf {
            Leaf::Field { name, .. } => Some(name.clone()),
            Leaf::Bound { .. } => None,
        })
        .collect();
    let noun = if fields.len() == 1 { "field" } else { "fields" };
    let list = quoted_field_list(&fields);
    let all_missing = resolved.causes.iter().all(|cause| {
        matches!(
            &cause.leaf,
            Leaf::Field {
                issue: FieldIssue::Missing,
                ..
            }
        )
    });
    let context = &resolved.context;
    if all_missing {
        format!("missing {noun} {list} on context `{context}`")
    } else {
        format!("accessor trait `HasField` with {noun} {list} is not implemented for `{context}`")
    }
}

/// The note body for one root cause: a short lead naming the leaf, then its dependency chain. The
/// specific fix (derive `HasField`) is not repeated per note — it rides in one `help` — so every
/// note is the same terse "required through this chain" form.
fn cause_note(cause: &Cause) -> String {
    let subject = match &cause.leaf {
        Leaf::Field { name, .. } => format!("field `{name}`"),
        Leaf::Bound { summary } => format!("`{summary}`"),
    };
    format!(
        "{subject} is required through this dependency chain:\n{}",
        render_dependency_tree(&cause.tree),
    )
}

/// The `= note:` subdiagnostic per root cause, each carrying that cause's dependency tree — the
/// sub-messages that replace rustc's own obligation-chain notes.
fn tree_notes(causes: &[Cause]) -> Vec<Subdiag> {
    causes
        .iter()
        .map(|cause| Subdiag {
            level: Level::Note,
            messages: vec![(DiagMessage::Str(cause_note(cause).into()), Style::NoStyle)],
            span: MultiSpan::new(),
        })
        .collect()
}

/// The distinct types that need a `#[derive(HasField)]`, in first-seen order — one per present or
/// `Deref`-reachable field (a `Deref`-reachable field points at its target, the type that must
/// actually derive). A genuinely missing field, or a non-field leaf, contributes none.
fn derive_targets(causes: &[Cause]) -> Vec<String> {
    let mut targets: Vec<String> = Vec::new();
    for cause in causes {
        let Leaf::Field { owner, issue, .. } = &cause.leaf else {
            continue;
        };
        let target = match issue {
            FieldIssue::Missing => continue,
            FieldIssue::Present => owner,
            FieldIssue::PresentViaDeref { target } => target,
        };
        if !targets.iter().any(|t| t == target) {
            targets.push(target.clone());
        }
    }
    targets
}

/// Build the wholesale replacement for an all-field resolution: a root-cause-first header carrying
/// the compiler's `E0277` code and its caret on the wiring entry, a `help` naming each type that
/// must derive `HasField`, and one terse tree note per field.
fn render_field_replacement(resolved: &Resolved, span: Span) -> DiagInner {
    let mut diag = DiagInner::new(Level::Error, field_header(resolved));
    diag.code = Some(E0277);
    diag.span = MultiSpan::from_span(span);

    for target in derive_targets(&resolved.causes) {
        let help = format!("make sure that `#[derive(HasField)]` is used for `{target}`");
        diag.children.push(Subdiag {
            level: Level::Help,
            messages: vec![(DiagMessage::Str(help.into()), Style::NoStyle)],
            span: MultiSpan::new(),
        });
    }

    diag.children.extend(tree_notes(&resolved.causes));
    diag
}

impl Emitter for CgpEmitter {
    fn emit_diagnostic(&mut self, mut diag: DiagInner) {
        // A resolvable wiring failure is transformed to its dependency tree(s). An all-field cause
        // is replaced wholesale with a clean, tree-first diagnostic; any other cause keeps rustc's
        // own main message and only swaps its sub-notes for the tree. Everything else falls back to
        // the in-place text rewrite.
        if let Some((resolved, span)) = self.try_resolve(&diag) {
            if resolved
                .causes
                .iter()
                .all(|cause| matches!(cause.leaf, Leaf::Field { .. }))
            {
                self.inner
                    .emit_diagnostic(render_field_replacement(&resolved, span));
            } else {
                rewrite_messages(&mut diag.messages, &self.names);
                diag.children = tree_notes(&resolved.causes);
                // Drop rustc's structured suggestions along with its notes — for a use-site
                // failure that includes the misleading "use associated function syntax instead".
                diag.suggestions = rustc_errors::Suggestions::Enabled(vec![]);
                self.inner.emit_diagnostic(diag);
            }
            return;
        }

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
