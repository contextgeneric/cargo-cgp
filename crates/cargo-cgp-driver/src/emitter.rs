//! The diagnostic-rewriting emitter the driver installs.
//!
//! This is the compiler-side seam of the message transform. It replaces the session's JSON
//! emitter with one that acts on each diagnostic before handing it to a real [`JsonEmitter`],
//! so the transformed result reaches cargo — and the front-end — already shaped, both in the
//! structured `children` and in the `rendered` field the JSON emitter regenerates from them.
//! It transforms a resolvable CGP wiring failure — a diagnostic on a `check_components!` entry, or
//! a broken consumer-method call (`E0599`) at its use site — into its root-cause dependency tree(s),
//! recovered from the compiler by [`resolve`](crate::resolve).
//!
//! The transform has two halves. The **main message** is rewritten only when it is identified
//! as a class of CGP error, and the rewrite then restates the same fact readably and stamps it
//! with the class's [CGP error code](cargo_cgp_error_processing::code) — an unsatisfied
//! `CanUseComponent` bound (or a broken consumer call) becomes `[CGP-E001] the consumer trait
//! `CanCalculateArea` is not implemented for context `Rectangle``, an unsatisfied
//! `IsProviderFor` bound the `[CGP-E002]` provider form. The diagnostic's own Rust code
//! (`E0277`, `E0599`) is always kept. A main message that is *not* a CGP class — an ordinary
//! bound like `f64: Eq` that the next-gen solver already descended to — stays rustc's own.
//! The **sub-messages** are replaced in either case: each recovered root cause becomes one
//! `note` naming the leaf (`root cause: missing field `height` on `Rectangle``, omitted when
//! the kept header already names that bound) followed by its rendered dependency chain, and a
//! `help` names each type that needs a `#[derive(HasField)]`. Everything the resolver cannot
//! handle falls back to a **text rewrite** of the compiler's own diagnostic, renaming CGP
//! wiring messages via the rustc-free [`rewrite`](cargo_cgp_error_processing::rewrite) module.
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

use cargo_cgp_error_processing::code::CONSUMER_TRAIT_UNIMPLEMENTED;
use cargo_cgp_error_processing::render_dependency_tree;
use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, parse_trait_bound, rewrite_message, rewrite_required_for, rewrite_trait_bound,
};
use rustc_errors::codes::E0599;
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

    /// Rewrite every recognized CGP wiring message in `diag`, in place — the fallback text
    /// transform for a diagnostic the typed resolver declined. The primary header takes the
    /// full rewrite (including the coded main-message forms); the children take only the
    /// obligation-chain rename, since a CGP error code belongs on a main message and never on
    /// a sub-message. A message that is not a wiring form is left untouched, and the name map
    /// is forced only when some message is actually rewritten.
    fn rewrite(&self, diag: &mut DiagInner) {
        rewrite_messages(&mut diag.messages, &self.names, rewrite_message);
        for child in &mut diag.children {
            rewrite_messages(&mut child.messages, &self.names, rewrite_required_for);
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

/// A `and`-joined, back-quoted list: `\`x\``, `\`x\` and \`y\``, or `\`x\`, \`y\`, and \`z\``.
fn quoted_list(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("`{item}`")).collect();
    match quoted.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The `[CGP-E001]` main message for a resolved failure: the consumer trait(s) the context
/// fails to implement, taken from the typed resolution — which keys each component marker by
/// its full path — so two same-named components in different modules can never be confused,
/// as they could be by the text lookup's bare-name match.
fn consumer_header(resolved: &Resolved) -> String {
    let (noun, verb) = if resolved.consumers.len() == 1 {
        ("trait", "is")
    } else {
        ("traits", "are")
    };
    format!(
        "[{CONSUMER_TRAIT_UNIMPLEMENTED}] the consumer {noun} {list} {verb} not implemented for context `{context}`",
        list = quoted_list(&resolved.consumers),
        context = resolved.context,
    )
}

/// The one root-cause lead line for a leaf — what the note names before the dependency chain.
/// A genuinely missing field is said plainly (without a `context` qualifier, since `HasField`
/// can land on any struct); a present-but-underived field is worded as the unimplemented
/// accessor, with the fix (the derive) carried by a separate `help`; any other leaf restates
/// its unmet bound.
fn root_cause_lead(leaf: &Leaf) -> String {
    match leaf {
        Leaf::Field {
            name,
            owner,
            issue: FieldIssue::Missing,
        } => format!("missing field `{name}` on `{owner}`"),
        Leaf::Field { name, owner, .. } => {
            format!(
                "accessor trait `HasField` with field `{name}` is not implemented for `{owner}`"
            )
        }
        Leaf::Bound { summary } => format!("the trait bound `{summary}` is not satisfied"),
    }
}

/// The note body for one root cause: the `root cause:` lead naming the leaf, then the rendered
/// dependency chain nested beneath its heading. When the diagnostic's kept main message already
/// states the leaf bound (`header_bound`), the lead would only repeat it, so the note carries
/// the chain alone.
fn cause_note(cause: &Cause, header_bound: Option<&str>) -> String {
    let chain = render_dependency_tree(&cause.tree);
    if let (Some(bound), Leaf::Bound { summary }) = (header_bound, &cause.leaf)
        && summary == bound
    {
        return format!("this is required through the dependency chain:\n{chain}");
    }
    let indented: String = chain
        .lines()
        .map(|line| format!("    {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "root cause: {}\nthis is required through the dependency chain:\n{indented}",
        root_cause_lead(&cause.leaf),
    )
}

/// The `= note:` subdiagnostic per root cause, each carrying that cause's dependency tree — the
/// sub-messages that replace rustc's own obligation-chain notes. `header_bound` is the bound the
/// kept main message states, if any, so a note does not restate it as its root cause.
fn tree_notes(causes: &[Cause], header_bound: Option<&str>) -> Vec<Subdiag> {
    causes
        .iter()
        .map(|cause| Subdiag {
            level: Level::Note,
            messages: vec![(
                DiagMessage::Str(cause_note(cause, header_bound).into()),
                Style::NoStyle,
            )],
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

/// The `help` subdiagnostics naming each type that must derive `HasField`, one per distinct
/// derive target of the resolved causes.
fn derive_helps(causes: &[Cause]) -> Vec<Subdiag> {
    derive_targets(causes)
        .into_iter()
        .map(|target| {
            let help = format!("make sure that `#[derive(HasField)]` is used for `{target}`");
            Subdiag {
                level: Level::Help,
                messages: vec![(DiagMessage::Str(help.into()), Style::NoStyle)],
                span: MultiSpan::new(),
            }
        })
        .collect()
}

/// The text of the diagnostic's main message, when it is a plain string.
fn main_message_text(diag: &DiagInner) -> Option<&str> {
    match diag.messages.first() {
        Some((DiagMessage::Str(text), _)) => Some(text),
        _ => None,
    }
}

impl CgpEmitter {
    /// The rewritten, `[CGP-Exxx]`-coded main message for a resolved failure — or `None` when
    /// the original main message is not an identified CGP error class and must be kept (an
    /// ordinary bound such as `f64: Eq` the solver already descended to). An unsatisfied
    /// `CanUseComponent` bound and a consumer-method `E0599` (whose text names no wiring
    /// trait) are both worded from the typed resolution, whose full-path marker keys make the
    /// consumer name exact; an unsatisfied `IsProviderFor` bound rewrites by its text, since
    /// the resolution does not carry the provider-side names.
    fn categorized_header(&self, diag: &DiagInner, resolved: &Resolved) -> Option<String> {
        if resolved.consumers.is_empty() {
            return None;
        }
        if let Some(text) = main_message_text(diag) {
            if let Some(parsed) = parse_trait_bound(text)
                && parsed.trait_name == "CanUseComponent"
            {
                return Some(consumer_header(resolved));
            }
            if let Some(rewritten) = rewrite_trait_bound(text, &self.names) {
                return Some(rewritten);
            }
        }
        if diag.code == Some(E0599) {
            return Some(consumer_header(resolved));
        }
        None
    }

    /// Transform a resolved wiring failure in place: rewrite the main message when it is an
    /// identified CGP class (keeping the diagnostic's own Rust code either way), and replace
    /// the sub-messages with the derive `help`s and one root-cause note per cause.
    fn transform_resolved(&self, diag: &mut DiagInner, resolved: &Resolved, span: Span) {
        // The bound the main message states, used two ways: a kept header's bound is not
        // restated as a note's root cause, and a rewritten header makes it moot.
        let header_bound = match self.categorized_header(diag, resolved) {
            Some(header) => {
                diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
                // Re-aim the caret at the failing entry alone: the original span labels
                // restate the replaced message, so they no longer apply.
                diag.span = MultiSpan::from_span(span);
                None
            }
            None => main_message_text(diag)
                .and_then(parse_trait_bound)
                .map(|parsed| parsed.bound.to_owned()),
        };

        let mut children = derive_helps(&resolved.causes);
        children.extend(tree_notes(&resolved.causes, header_bound.as_deref()));
        diag.children = children;
        // Drop rustc's structured suggestions along with its notes — for a use-site failure
        // that includes the misleading "use associated function syntax instead".
        diag.suggestions = rustc_errors::Suggestions::Enabled(vec![]);
    }
}

impl Emitter for CgpEmitter {
    fn emit_diagnostic(&mut self, mut diag: DiagInner) {
        // A resolvable wiring failure is transformed around its dependency tree(s): the main
        // message is rewritten (and coded) when it is an identified CGP class, and the
        // sub-messages become the root-cause notes. Everything else falls back to the in-place
        // text rewrite.
        if let Some((resolved, span)) = self.try_resolve(&diag) {
            self.transform_resolved(&mut diag, &resolved, span);
        } else {
            self.rewrite(&mut diag);
        }
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

/// Rewrite each plain-string message in place through `rewrite`, leaving its style and any
/// Fluent message untouched. The rewrite function ([`rewrite_message`] for a main message,
/// [`rewrite_required_for`] for sub-messages) consults the name map only for a message that
/// parses as a CGP wiring form.
fn rewrite_messages<S>(
    messages: &mut [(DiagMessage, S)],
    names: &ComponentNameMap,
    rewrite: fn(&str, &ComponentNameMap) -> Option<String>,
) {
    for (message, _) in messages.iter_mut() {
        if let DiagMessage::Str(text) = message
            && let Some(rewritten) = rewrite(text, names)
        {
            *message = DiagMessage::Str(Cow::Owned(rewritten));
        }
    }
}
