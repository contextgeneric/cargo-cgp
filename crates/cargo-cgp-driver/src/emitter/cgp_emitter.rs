//! The wrapping [`Emitter`] that transforms CGP diagnostics before delegating.

use std::path::Path;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, rewrite_message, rewrite_required_for, wiring_overflow_help,
};
use cargo_cgp_error_processing::{
    DedupLedger, DiagKind, OrphanConflict, Resolved, cause_signature, is_method_bounds_text,
    mentions_orphan_param_text, orphan_conflict_help, plan_orphan_conflict, plan_resolved,
    plan_wiring_conflict, wiring_conflict_help,
};
use rustc_errors::codes::{E0117, E0119, E0210, E0271, E0275, E0277, E0599};
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::timings::TimingRecord;
use rustc_errors::{DiagInner, DiagMessage, Level, MultiSpan, Style, Suggestions};
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;
use crate::emitter::edit::{
    diag_kind, diagnostic_spans, is_question_mark_cascade, main_message_text,
    mentions_hasfield_impls, mentions_wiring, message_signature, postprocess_messages,
    postprocess_multispan, replace_header, rewrite_messages, strip_method_probe_advice, subdiag,
};
use crate::resolve::{self, ConflictAction, ConflictTrait, ResolveCache};

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
    /// The ledger of CGP diagnostics already emitted this compilation, so a wiring mistake that
    /// surfaces at many sites is shown once and its identical re-reports are suppressed (see
    /// [`emit_diagnostic`](CgpEmitter::emit_diagnostic)). The keys — the recovered root cause for
    /// a resolved diagnostic, the rendered text for a declined-but-rewritten one, and the coded
    /// header — live with the ledger in the rustc-free crate.
    dedup: DedupLedger,
    /// Memoization of the typed resolver's walk, so a wiring mistake re-reported at many sites is
    /// resolved once and reused rather than re-walked per diagnostic (see
    /// [`ResolveCache`] and `docs/implementation/cached-dependency-resolution.md`). Keyed at
    /// **every node** on the region-erased obligation and its context, valued by that node's owned
    /// rustc-free sub-result, so entries persist for the whole compilation like
    /// [`names`](Self::names) and [`dedup`](Self::dedup).
    resolve_cache: ResolveCache,
    /// The primary spans of every CGP wiring failure this emitter has recognized this
    /// compilation. Used to drop a downstream `?`-operator cascade
    /// ([`is_question_mark_cascade`]) that lands on the same expression: once a wiring bound on
    /// `expr` fails, the type of `expr?` cannot be resolved, so rustc adds a `Try`/`FromResidual`
    /// error at the same span that restates the wiring failure and dumps the unresolved projected
    /// type — noise over the CGP error already shown. rustc emits the wiring failure before its
    /// cascade, so the span is recorded by the time the cascade arrives.
    cgp_spans: Vec<Span>,
}

impl<E> CgpEmitter<E> {
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            names: ComponentNameMap::new(build_name_map_from_tls),
            dedup: DedupLedger::new(),
            resolve_cache: ResolveCache::new(),
            cgp_spans: Vec::new(),
        }
    }

    /// Record the primary spans of a recognized CGP wiring failure, so a later `?`-operator
    /// cascade on the same expression can be suppressed. Called for every diagnostic the emitter
    /// transforms, before de-duplication, so even a re-report that is itself dropped still anchors
    /// its cascade.
    fn record_cgp_spans(&mut self, diag: &DiagInner) {
        self.cgp_spans
            .extend(diag.span.primary_spans().iter().copied());
    }

    /// Whether `diag` sits on an expression where a CGP wiring failure was already reported — a
    /// primary span overlapping one recorded in [`cgp_spans`](Self::cgp_spans). Paired with
    /// [`is_question_mark_cascade`] to drop the `?`-operator errors that cascade from that failure.
    fn overlaps_cgp_failure(&self, diag: &DiagInner) -> bool {
        diag.span
            .primary_spans()
            .iter()
            .any(|span| self.cgp_spans.iter().any(|seen| seen.overlaps(*span)))
    }

    /// Rewrite every recognized CGP wiring message in `diag`, in place — the first fallback
    /// text pass for a diagnostic the typed resolver declined. The primary header takes the
    /// full rewrite (including the coded main-message forms); the children take only the
    /// obligation-chain rename, since a CGP error code belongs on a main message and never on
    /// a sub-message. A message that is not a wiring form is left untouched, and the name map
    /// is forced only when some message is actually rewritten.
    fn rewrite(&self, diag: &mut DiagInner) -> bool {
        let mut changed = rewrite_messages(&mut diag.messages, &self.names, rewrite_message);
        for child in &mut diag.children {
            changed |= rewrite_messages(&mut child.messages, &self.names, rewrite_required_for);
        }
        changed
    }

    /// Post-process a diagnostic after transforming it — the final cleanup pass, over every
    /// message and span label of the diagnostic and its children. It strips CGP path prefixes,
    /// resugars `Symbol!` and `Path!`, and rewords an unmet `HasField` bound. Whether the
    /// context implements `HasField` for any field is a fact of the whole diagnostic (the
    /// "similar impl" landmark can sit far from the clause), so it is decided once up front and
    /// passed into each per-message rewrite.
    /// `bare_paths` distinguishes a rewritten diagnostic from a resugaring fallback: a message the
    /// tool constructed (a wiring-conflict rewrite or a typed resolution) shows a bare `@…` path,
    /// while an un-rewritten fallback keeps the `Path!(@…)` macro form.
    fn postprocess(&self, diag: &mut DiagInner, bare_paths: bool) {
        let has_field_impls = mentions_hasfield_impls(diag);
        postprocess_messages(&mut diag.messages, has_field_impls, bare_paths);
        postprocess_multispan(&mut diag.span, has_field_impls, bare_paths);
        for child in &mut diag.children {
            postprocess_messages(&mut child.messages, has_field_impls, bare_paths);
            postprocess_multispan(&mut child.span, has_field_impls, bare_paths);
        }
    }

    /// Recognize `diag` as a duplicate-key wiring conflict — the `E0119` coherence error a
    /// duplicate `delegate_components!` or `cgp_namespace!` entry produces — returning the action
    /// to take, or `None` when it is not one. The message text routes the shape (a redundant
    /// `IsProviderFor` half is suppressed, a `DelegateComponent` half rewritten, and any other
    /// trait tried as a `cgp_namespace!` conflict); the typed classifier verifies genuine CGP
    /// impls sit at the caret before anything fires.
    fn wiring_conflict(&self, diag: &DiagInner) -> Option<ConflictAction> {
        if diag.code != Some(E0119) {
            return None;
        }
        let message = main_message_text(diag)?;
        // `IsProviderFor` is checked first: a `DelegateComponent` conflict names only
        // `DelegateComponent`, while its companion names `IsProviderFor`. A message naming
        // neither can still be a `cgp_namespace!` conflict on the user's own namespace trait,
        // which the classifier recognizes by the impls at the carets.
        let variant = if message.contains("IsProviderFor") {
            ConflictTrait::IsProviderFor
        } else if message.contains("DelegateComponent") {
            ConflictTrait::Delegate
        } else {
            ConflictTrait::Other
        };
        let primary_span = diag.span.primary_span()?;
        let label_spans: Vec<Span> = diag
            .span
            .span_labels()
            .into_iter()
            .map(|label| label.span)
            .collect();
        rustc_middle::ty::tls::with_opt(|tcx| {
            resolve::classify_wiring_conflict(tcx?, variant, primary_span, &label_spans)
        })
    }

    /// Recognize `diag` as an orphan-rule namespace registration — the `E0210` (or its sibling
    /// `E0117`) the orphan rule produces when a crate registers wiring into a *foreign* namespace
    /// keyed on a *foreign* key, so the generated `impl Namespace<_> for Key` has no local type.
    /// Returns the recovered conflict, or `None` when it is not one. A cheap text pre-filter — the
    /// machinery parameter (`__Components__` / `__Table__`) the `E0210` message names — gates the
    /// impl scan; the rarer `E0117`, whose message names no parameter, is left to the typed
    /// classifier alone, which confirms a foreign namespace trait sits at the caret before firing.
    fn orphan_conflict(&self, diag: &DiagInner) -> Option<OrphanConflict> {
        if !matches!(diag.code, Some(E0210) | Some(E0117)) {
            return None;
        }
        if diag.code == Some(E0210)
            && !main_message_text(diag).is_some_and(mentions_orphan_param_text)
        {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        rustc_middle::ty::tls::with_opt(|tcx| resolve::classify_orphan_conflict(tcx?, primary_span))
    }

    /// Resolve `diag`'s failure to its root-cause dependency tree(s), or `None` when the resolver
    /// cannot trace it to a CGP component failure (so the caller falls back to the in-place text
    /// rewrite). A candidate is any diagnostic that mentions a CGP wiring trait, plus every `E0271`,
    /// `E0277`, and *method-bounds* `E0599` — because a failure *not* worded in CGP terms can still
    /// be a consequence of a CGP component failing (a manual `impl` that forwards to a wired method, a
    /// downstream trait bound that needs it), and [`resolve`] traces the dependency chain to find
    /// out. It yields `None` for everything whose chain does not reach a CGP cause. Returns the
    /// primary span alongside the resolution so the caret can be re-aimed at the entry.
    ///
    /// The `E0599` arm is narrowed to the "the method `…` exists … but its trait bounds were not
    /// satisfied" shape — the consumer-method call the use-site anchor handles. A *resolution*-class
    /// `E0599` (`no variant named …`, `no associated item …`) is not a wiring failure and, worse, is
    /// emitted *during* type lowering / `predicates_of`, while that query is mid-flight: running the
    /// resolver's trait solver on it re-forces an emitting query and re-enters the already-held
    /// `DiagCtxt` lock, aborting the compiler (`lock was already held`). Declining such an `E0599`
    /// before any solving both keeps the tool from crashing and is correct, since the resolver has
    /// nothing to say about a name-resolution error. (`E0271`/`E0277` are trait-solving failures
    /// reported after collection, where the queries the solver forces are already cached.)
    fn try_resolve(&self, diag: &DiagInner) -> Option<(Resolved, Span, bool)> {
        // A method-bounds `E0599` (not a resolution-class one) is the only `E0599` the resolver
        // handles; see the re-entrancy note above.
        let e0599_method_bounds =
            diag.code == Some(E0599) && main_message_text(diag).is_some_and(is_method_bounds_text);
        if !mentions_wiring(diag)
            && !matches!(diag.code, Some(E0271) | Some(E0277))
            && !e0599_method_bounds
        {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        let cache = &self.resolve_cache;
        let (resolved, at_call) = rustc_middle::ty::tls::with_opt(|tcx| {
            let tcx = tcx?;
            let spans = diagnostic_spans(diag);
            // Prefer the check-entry anchor (an obligation recovered from the check impl at the
            // caret). Failing that, the impl-site anchor recovers the exact failing obligation —
            // with its concrete component parameters — from a hand-written `impl … for Context`
            // block the failure surfaces inside, which is more precise than the use-site re-check.
            // Failing that, the wrapper-chain anchor handles a wrapper implemented on a *foreign*
            // type holding the context (`impl … for Router<Arc<MockApp>>`), whose CGP consumer
            // failure sits several `where`-clause hops down. Failing all — a use-site failure such
            // as a consumer-method call, whose obligation no check impl carries — recover the
            // context from the diagnostic's spans.
            let resolved = resolve::resolve_check_failure(tcx, cache, primary_span)
                .or_else(|| resolve::resolve_impl_site(tcx, cache, &spans))
                .or_else(|| resolve::resolve_wrapper_chain(tcx, cache, &spans))
                .or_else(|| resolve::resolve_use_site(tcx, cache, &spans))
                // A namespace-joined context's wiring lives in the namespace, not its own
                // `DelegateComponent` impls, so the per-component re-check above finds nothing;
                // anchoring on the consumer trait the diagnostic names and walking through the
                // namespace recovers it.
                .or_else(|| resolve::resolve_use_site_consumer(tcx, cache, &spans));
            if let Some(resolved) = resolved {
                return Some((resolved, false));
            }
            // The last resort re-reads the failing *call expression* itself — the anchor for a
            // consumer-method `E0277` whose spans never touch the context's definition (a
            // `Code`-dispatched handler pipeline that matches unconditionally). A resolution from
            // here is flagged, so the header is worded from the consumer the call needs rather
            // than from whichever provider bound rustc's headline stopped on.
            Some((resolve::resolve_call_site(tcx, cache, &spans)?, true))
        })?;
        Some((resolved, primary_span, at_call))
    }

    /// Transform a resolved wiring failure in place from the rustc-free [`plan_resolved`]: replace
    /// the main message when the plan carries a coded header (re-aiming the caret at the failing
    /// entry), then replace the sub-messages with the plan's derive `help`s and one root-cause
    /// note per cause, dropping rustc's own suggestions. A resolution anchored at the call
    /// expression (`at_call`) plans as a use-site failure whatever its rustc code, so its header
    /// names the consumer the call needs — except a genuine field-type mismatch, whose `E0271`
    /// class words the more specific `[CGP-E003]` form.
    fn transform_resolved(
        &self,
        diag: &mut DiagInner,
        resolved: &Resolved,
        span: Span,
        at_call: bool,
    ) {
        let kind = match diag_kind(diag) {
            kind if at_call && kind != DiagKind::FieldMismatch => DiagKind::MethodNotFound,
            kind => kind,
        };
        let plan = plan_resolved(kind, main_message_text(diag), resolved, &self.names);

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
        // A duplicate-key coherence conflict (E0119) is handled as one logical error: the
        // redundant `IsProviderFor` half of the pair is dropped, and the `DelegateComponent` half
        // is reworded to name the colliding key(s), keeping rustc's two carets.
        if let Some(action) = self.wiring_conflict(&diag) {
            match action {
                ConflictAction::Suppress => return,
                ConflictAction::Rewrite(conflict) => {
                    replace_header(&mut diag, plan_wiring_conflict(&conflict));
                    // A redirect collision carries its fix as a `help`, kept out of the header.
                    if let Some(help) = wiring_conflict_help(&conflict) {
                        diag.children.push(subdiag(Level::Help, help));
                    }
                    // A rewritten diagnostic: bare `@…` paths.
                    self.postprocess(&mut diag, true);
                    self.record_cgp_spans(&diag);
                    self.inner.emit_diagnostic(diag);
                    return;
                }
            }
        }
        // An orphan-rule namespace registration (E0210/E0117): a crate registering wiring into a
        // foreign namespace keyed on a foreign key. Reword the raw coherence error — which names
        // the machinery parameter and frames a CGP wiring decision as a bare coherence rule — into
        // a `[CGP-E011]` header naming the namespace and key, re-aiming the caret at the offending
        // macro alone (its "uncovered type parameter" label no longer applies), and carrying the
        // ownership-based fix in a `help`.
        if let Some(conflict) = self.orphan_conflict(&diag)
            && let Some(primary_span) = diag.span.primary_span()
        {
            replace_header(&mut diag, plan_orphan_conflict(&conflict));
            diag.span = MultiSpan::from_span(primary_span);
            diag.children
                .push(subdiag(Level::Help, orphan_conflict_help(&conflict)));
            // A rewritten diagnostic: bare `@…` paths (a path key renders without the `Path!`
            // wrapper).
            self.postprocess(&mut diag, true);
            self.record_cgp_spans(&diag);
            self.inner.emit_diagnostic(diag);
            return;
        }
        // A resolvable wiring failure is transformed around its dependency tree(s); when the
        // resolver declines, the wiring-message rename runs as the first fallback pass. A resolved
        // failure also yields its span-independent cause signature, for the de-duplication below.
        let (rewritten, cause_sig) =
            if let Some((resolved, span, at_call)) = self.try_resolve(&diag) {
                let sig = cause_signature(&resolved);
                self.transform_resolved(&mut diag, &resolved, span, at_call);
                (true, Some(sig))
            } else {
                // A CGP consumer-method `E0599` the resolver declined still carries rustc's
                // method-probe advice — the associated-function framing and the "use associated
                // function syntax instead" suggestion, both artifacts of the provider's `self`-less
                // methods, the second actively wrong. Strip that noise so the unmet wiring bound
                // the diagnostic also names is not outranked by it.
                if diag.code == Some(E0599)
                    && main_message_text(&diag).is_some_and(is_method_bounds_text)
                    && mentions_wiring(&diag)
                {
                    strip_method_probe_advice(&mut diag);
                }
                let changed = self.rewrite(&mut diag);
                // A rewritten wiring overflow (`E0275`, now a `[CGP-E010]` header) drops the
                // note pointing at the generated `__Check…` trait — a name the user never wrote,
                // whose location the kept caret already covers — and carries the fix in a `help`.
                if changed && diag.code == Some(E0275) {
                    diag.children.retain(|child| {
                        !child.messages.iter().any(|(message, _)| {
                            matches!(
                                message,
                                DiagMessage::Str(text)
                                    if text.contains("__Check") || text.contains("__CanUse")
                            )
                        })
                    });
                    diag.children
                        .push(subdiag(Level::Help, wiring_overflow_help()));
                }
                (changed, None)
            };
        // Post-process the result either way, so no raw CGP construct leaks. A typed resolution
        // or a text rewrite constructs the message, so its paths render bare (`@…`); a diagnostic
        // the tool left untouched keeps the `Path!(@…)` resugaring form.
        self.postprocess(&mut diag, rewritten);
        if rewritten {
            // Remember where this wiring failure landed, so a `?`-operator cascade on the same
            // expression can be dropped below. Recorded before de-duplication, so a re-report that
            // is itself suppressed still anchors its cascade.
            self.record_cgp_spans(&diag);
        } else if is_question_mark_cascade(&diag) && self.overlaps_cgp_failure(&diag) {
            // A downstream `?`-operator cascade of a CGP wiring failure already reported at this
            // expression: it restates the failure in `Try` terms and dumps the unresolved projected
            // type, adding nothing. Drop it (cargo re-counts emitted diagnostics, so the "N errors"
            // summary stays honest). A `?` error with no CGP failure on its expression is untouched.
            return;
        }
        // Cross-diagnostic de-duplication. CGP wiring is lazy, so one mistake surfaces as the same
        // error at many sites — the `check_components!` entry, every hand-written `impl` that
        // references the broken component, and each call. A transformed diagnostic whose signature
        // the ledger has already recorded is such a re-report, so it is suppressed and only the
        // first occurrence is shown; the key scheme lives with the [`DedupLedger`]. Only the tool's
        // own transformed diagnostics are de-duplicated; an untouched `rustc` error always passes
        // through. cargo re-counts the diagnostics the emitter produces, so a suppressed re-report
        // drops out of its "N errors" summary as well, keeping the count consistent.
        if rewritten
            && self.dedup.check_and_record(
                cause_sig.as_deref(),
                || message_signature(&diag),
                main_message_text(&diag),
            )
        {
            return;
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

    fn supports_color(&self) -> bool {
        self.inner.supports_color()
    }
}
