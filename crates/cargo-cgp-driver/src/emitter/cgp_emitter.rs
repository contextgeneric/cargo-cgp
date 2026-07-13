//! The wrapping [`Emitter`] that transforms CGP diagnostics before delegating.

use std::path::Path;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, rewrite_message, rewrite_required_for,
};
use cargo_cgp_error_processing::{Resolved, plan_resolved};
use rustc_errors::codes::E0599;
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::timings::TimingRecord;
use rustc_errors::{DiagInner, DiagMessage, Level, MultiSpan, Style, Suggestions};
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;
use crate::emitter::edit::{
    diag_kind, diagnostic_spans, main_message_text, mentions_hasfield_impls, mentions_wiring,
    postprocess_messages, postprocess_multispan, rewrite_messages, subdiag,
};
use crate::resolve;

/// The wrapping [`Emitter`] that transforms CGP diagnostics before delegating to the real
/// inner emitter. Generic over the inner emitter `E` so the driver can wrap whichever the
/// compiler's default would build for the active error format — a `JsonEmitter` or an
/// `AnnotateSnippetEmitter` — and render like vanilla `rustc` in either.
pub struct CgpEmitter<E> {
    inner: E,
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

impl<E> CgpEmitter<E> {
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            names: ComponentNameMap::new(build_name_map_from_tls),
        }
    }

    /// Rewrite every recognized CGP wiring message in `diag`, in place — the first fallback
    /// text pass for a diagnostic the typed resolver declined. The primary header takes the
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

    /// Post-process a diagnostic after transforming it — the final cleanup pass, over every
    /// message and span label of the diagnostic and its children. It strips CGP path prefixes,
    /// resugars `Symbol!` and `Path!`, and rewords an unmet `HasField` bound. Whether the
    /// context implements `HasField` for any field is a fact of the whole diagnostic (the
    /// "similar impl" landmark can sit far from the clause), so it is decided once up front and
    /// passed into each per-message rewrite.
    fn postprocess(&self, diag: &mut DiagInner) {
        let has_field_impls = mentions_hasfield_impls(diag);
        postprocess_messages(&mut diag.messages, has_field_impls);
        postprocess_multispan(&mut diag.span, has_field_impls);
        for child in &mut diag.children {
            postprocess_messages(&mut child.messages, has_field_impls);
            postprocess_multispan(&mut child.span, has_field_impls);
        }
    }

    /// Resolve `diag`'s CGP wiring failure to its root-cause dependency tree(s), or `None` when
    /// this is not a resolvable wiring diagnostic (so the caller falls back to the in-place text
    /// rewrite). A candidate is any diagnostic whose messages mention a CGP wiring trait, plus
    /// every method `E0599`; [`resolve`] does the typed work and yields `None` for everything it
    /// cannot fully resolve. Returns the primary span alongside the resolution so the
    /// field-replacement path can re-aim the caret at the entry.
    fn try_resolve(&self, diag: &DiagInner) -> Option<(Resolved, Span)> {
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

    /// Transform a resolved wiring failure in place from the rustc-free [`plan_resolved`]: replace
    /// the main message when the plan carries a coded header (re-aiming the caret at the failing
    /// entry), then replace the sub-messages with the plan's derive `help`s and one root-cause
    /// note per cause, dropping rustc's own suggestions.
    fn transform_resolved(&self, diag: &mut DiagInner, resolved: &Resolved, span: Span) {
        let plan = plan_resolved(
            diag_kind(diag),
            main_message_text(diag),
            resolved,
            &self.names,
        );

        if let Some(header) = plan.header {
            diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
            // Re-aim the caret at the failing entry alone: the original span labels restate the
            // replaced message, so they no longer apply.
            diag.span = MultiSpan::from_span(span);
        }

        let mut children = Vec::new();
        children.extend(
            plan.helps
                .into_iter()
                .map(|help| subdiag(Level::Help, help)),
        );
        children.extend(
            plan.notes
                .into_iter()
                .map(|note| subdiag(Level::Note, note)),
        );
        diag.children = children;
        // Drop rustc's structured suggestions along with its notes — for a use-site failure
        // that includes the misleading "use associated function syntax instead".
        diag.suggestions = Suggestions::Enabled(vec![]);
    }
}

impl<E: Emitter> Emitter for CgpEmitter<E> {
    fn emit_diagnostic(&mut self, mut diag: DiagInner) {
        // A resolvable wiring failure is transformed around its dependency tree(s); when the
        // resolver declines, the wiring-message rename runs as the first fallback pass.
        if let Some((resolved, span)) = self.try_resolve(&diag) {
            self.transform_resolved(&mut diag, &resolved, span);
        } else {
            self.rewrite(&mut diag);
        }
        // Post-process the result either way, so no raw CGP construct leaks.
        self.postprocess(&mut diag);
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

    fn supports_color(&self) -> bool {
        self.inner.supports_color()
    }
}
