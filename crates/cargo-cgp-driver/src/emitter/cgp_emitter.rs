//! The wrapping [`Emitter`] that transforms CGP diagnostics before delegating.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{DedupLedger, Origin};
use rustc_errors::DiagInner;
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::timings::TimingRecord;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;
use crate::emitter::attribute_splice::drop_attribute_splices;
use crate::emitter::buffer::{BufEntry, ResolvedEntry};
use crate::emitter::crate_facts::{CrateFacts, EMPTY_AMBIGUOUS_NAMES};
use crate::emitter::edit::{
    bound_candidate_lists, main_message_text, message_signature, postprocess_diag,
};
use crate::emitter::fallback::{render_or_untransformed, transform_or_original};
use crate::emitter::flush::render;
use crate::emitter::generated_names::drop_generated_names;
use crate::emitter::output::{Output, lock_output, register_output};
use crate::emitter::pipeline::{BufferEdit, Reshape, Verdict, apply_reshape};
use crate::emitter::stage::{
    CgpImplMisuseStage, MacrolessProviderStage, Resolution, ResolvedFailure, StructuralStage,
    UnimportedStage, consumer_trait_as_key, is_cgp_impl_sibling_entry, missing_use_provider,
    orphan_conflict, provider_trait_in_uses, question_mark_cascade, resolve_or_rewrite,
    shared_dispatch_method, undeclared_requirement, undeclared_trait, unresolved_wiring_overflow,
    wiring_conflict,
};
use crate::resolve::{ResolveCache, ResolveCtx};

/// The wrapping [`Emitter`] that transforms CGP diagnostics before delegating to the real
/// inner emitter. Generic over the inner emitter `E` so the driver can wrap whichever the
/// compiler's default would build for the active error format — a `JsonEmitter` or an
/// `AnnotateSnippetEmitter` — and render like vanilla `rustc` in either.
pub struct CgpEmitter<E: Emitter> {
    /// The compiler's own emitter and the diagnostics buffered for it, shared with the panic hook
    /// so a compiler crash can print the buffer before its report (see [`Output`]).
    output: Arc<Mutex<Output<E>>>,
    /// The source map the inner emitter renders against, kept here as well because
    /// [`Emitter::source_map`] lends it out, which a value behind the lock cannot be.
    source_map: Option<Arc<SourceMap>>,
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
    /// resolved once and reused rather than re-walked per diagnostic (see [`ResolveCache`] and
    /// `cgp-knowledge-base/cargo-cgp/implementation/cached-dependency-resolution.md`). Keyed at
    /// **every node** on the region-erased obligation and its context, valued by that node's owned
    /// rustc-free sub-result, so entries persist for the whole compilation like
    /// [`names`](Self::names) and [`dedup`](Self::dedup).
    resolve_cache: ResolveCache,
    /// The primary spans of every CGP wiring failure this emitter has recognized this
    /// compilation, written by the tails and read by stage 15 ([`question_mark_cascade`]) to drop
    /// a downstream `?`-operator cascade on the same expression. rustc emits the wiring failure
    /// before its cascade, so the span is recorded by the time the cascade arrives.
    cgp_spans: Vec<Span>,
    /// Stage 2, which owns the crate's `#[cgp_impl]` header mistakes once detected; the resolver's
    /// stage reads them to drop a check re-report of one.
    cgp_impl_misuse: CgpImplMisuseStage,
    /// Stage 9, which owns the structural mistakes already reported.
    structural: StructuralStage,
    /// Stages 12 and 16, which own the providers reported as written without `#[cgp_provider]`.
    macroless_provider: MacrolessProviderStage,
    /// Stages 13 and 14, which own the abstract types and the provider impls already reported.
    unimported: UnimportedStage,
    /// The crate facts the post-processing reads, decided once the compiler can answer.
    facts: CrateFacts,
}

impl<E: Emitter> CgpEmitter<E> {
    /// Wrap `inner`, which renders against `source_map`, and register the shared output for the
    /// panic hook on this thread.
    pub fn new(inner: E, source_map: Option<Arc<SourceMap>>) -> Self
    where
        E: 'static,
    {
        let output = Arc::new(Mutex::new(Output {
            inner,
            buffer: Vec::new(),
            unprovable_requirements: Vec::new(),
        }));
        register_output(&output);
        Self {
            output,
            source_map,
            names: ComponentNameMap::new(build_name_map_from_tls),
            dedup: DedupLedger::new(),
            resolve_cache: ResolveCache::new(),
            cgp_spans: Vec::new(),
            cgp_impl_misuse: CgpImplMisuseStage::default(),
            structural: StructuralStage::default(),
            macroless_provider: MacrolessProviderStage::default(),
            unimported: UnimportedStage::default(),
            facts: CrateFacts::default(),
        }
    }

    /// The shared output, locked.
    fn output(&self) -> MutexGuard<'_, Output<E>> {
        lock_output(&self.output)
    }

    /// Buffer `entry` for the flush.
    fn push(&self, entry: BufEntry) {
        self.output().buffer.push(entry);
    }

    /// Write `reshape` into `diag` and buffer the result: the tail every reshaping stage shares.
    /// The buffer edit goes first, then the wording and the span, the post-processing of a
    /// rewritten diagnostic, and the span record that lets a later `?` cascade on the same
    /// expression be dropped.
    fn reshape(&mut self, mut diag: DiagInner, reshape: Reshape) {
        let Reshape {
            plan,
            span,
            records_spans,
            buffer_edit,
        } = reshape;
        self.edit_buffer(buffer_edit);
        apply_reshape(&mut diag, plan, span);
        self.postprocess(&mut diag, Origin::Rewritten);
        if records_spans {
            self.record_cgp_spans(&diag);
        }
        self.push(BufEntry::Plain(Box::new(diag)));
    }

    /// Apply a reshape's edit of the diagnostics already buffered.
    fn edit_buffer(&self, edit: BufferEdit) {
        match edit {
            BufferEdit::None => {}
            BufferEdit::DropCgpImplSiblings {
                impl_span,
                macro_span,
            } => self
                .output()
                .buffer
                .retain(|entry| !is_cgp_impl_sibling_entry(entry, impl_span, macro_span)),
            BufferEdit::Remove(index) => {
                self.output().buffer.remove(index);
            }
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

    /// Post-process a diagnostic after transforming it ([`postprocess_diag`]). `origin` says
    /// whether the tool rewrote it; an untransformed one changes only where the chain recovers a
    /// CGP encoding (see [`Origin`]), and only in a crate that loads CGP, so a plain Rust error
    /// passes through exactly as rustc built it.
    fn postprocess(&self, diag: &mut DiagInner, origin: Origin) {
        if origin == Origin::Untransformed && !self.facts.uses_cgp() {
            return;
        }
        postprocess_diag(diag, origin, self.facts.ambiguous_names(origin));
    }
}

impl<E: Emitter> CgpEmitter<E> {
    /// Transform one diagnostic and buffer the result: the body of
    /// [`emit_diagnostic`](Emitter::emit_diagnostic), which runs it contained. `original` is the
    /// diagnostic as rustc built it, kept for the flush to fall back on.
    fn transform(&mut self, diag: DiagInner, original: &DiagInner) {
        // The compiler handle is read once for the whole stage list. Reading it runs no query, so a
        // stage that must not query on an early-phase diagnostic still decides that itself.
        rustc_middle::ty::tls::with_opt(|tcx| self.transform_with(tcx, diag, original));
    }

    /// [`transform`](Self::transform) with the compiler handle, when one is in scope.
    fn transform_with(
        &mut self,
        tcx: Option<TyCtxt<'_>>,
        mut diag: DiagInner,
        original: &DiagInner,
    ) {
        // The injected `--verbose` lists every implementor in a failed bound's help; cut the list
        // back to rustc's own length before anything else reads the diagnostic.
        bound_candidate_lists(&mut diag);
        let verdict = self
            .cgp_impl_misuse
            .run(tcx, &diag)
            .or_else(|| provider_trait_in_uses(tcx, &diag))
            .or_else(|| unresolved_wiring_overflow(&diag, &self.output().unprovable_requirements))
            .or_else(|| consumer_trait_as_key(tcx, &diag))
            .or_else(|| wiring_conflict(tcx, &diag))
            .or_else(|| orphan_conflict(tcx, &diag))
            .or_else(|| shared_dispatch_method(&diag, &self.output().buffer))
            .or_else(|| self.structural.run(tcx, &diag))
            .or_else(|| undeclared_trait(tcx, &diag))
            .or_else(|| missing_use_provider(tcx, &diag))
            .or_else(|| self.macroless_provider.run(tcx, &diag))
            .or_else(|| self.unimported.unimported_abstract_type(tcx, &diag))
            .or_else(|| match undeclared_requirement(tcx, &diag) {
                // The requirement's query runs first; only then is the buffer locked, to convert
                // the name-resolution errors of the abstract types it names.
                Some(detected) => self
                    .unimported
                    .requirement(detected, &mut lock_output(&self.output).buffer),
                None => Verdict::Continue,
            })
            .or_else(|| question_mark_cascade(&diag, &self.cgp_spans))
            .or_else(|| self.macroless_provider.consequence(&diag));
        match verdict {
            Verdict::Continue => {}
            Verdict::Drop => return,
            Verdict::Reshape(reshape) => return self.reshape(diag, reshape),
        }
        let cx = tcx.map(|tcx| ResolveCtx {
            tcx,
            cache: &self.resolve_cache,
        });
        let resolution = resolve_or_rewrite(cx, &mut diag, &self.names, &self.cgp_impl_misuse);
        self.finish(diag, original, resolution);
    }

    /// The final tail, after stage 17: post-process the result, drop the suggestions a macro made
    /// wrong, record a rewritten diagnostic's spans, de-duplicate it, and buffer it.
    fn finish(&mut self, mut diag: DiagInner, original: &DiagInner, resolution: Resolution) {
        let (rewritten, cause_sig, resolution, class_headers) = match resolution {
            Resolution::Drop => return,
            Resolution::Resolved(failure) => {
                let ResolvedFailure {
                    resolved,
                    cause_signature,
                    transformed,
                } = *failure;
                (
                    true,
                    Some(cause_signature),
                    // A failure whose headline gives way to its cause coalesces at flush with
                    // every failure it shares a root cause with; a mismatch or a kept rustc
                    // headline emits on its own. Every resolution defers its note to the flush,
                    // so all of them elide against one another.
                    Some((resolved, transformed.note, transformed.coalesces)),
                    transformed.headers,
                )
            }
            Resolution::Declined { rewritten } => (rewritten, None, None, Vec::new()),
        };
        // Post-process the result either way, so no raw CGP construct leaks. A typed resolution
        // or a text rewrite constructs the message, so its paths render bare (`@…`); a diagnostic
        // the tool left untouched keeps the `Path!(@…)` resugaring form.
        let origin = if rewritten {
            Origin::Rewritten
        } else {
            Origin::Untransformed
        };
        self.postprocess(&mut diag, origin);
        if self.facts.uses_cgp() {
            drop_attribute_splices(&mut diag);
            if let Some(source_map) = self.source_map.as_deref() {
                drop_generated_names(&mut diag, source_map);
            }
        }
        if rewritten {
            // Remember where this wiring failure landed, so a later `?`-operator cascade on the same
            // expression is dropped. Recorded before de-duplication, so a re-report that is itself
            // suppressed still anchors its cascade.
            self.record_cgp_spans(&diag);
        }
        // Cross-diagnostic de-duplication. CGP wiring is lazy, so one mistake surfaces as the same
        // error at many sites — the `check_components!` entry, every hand-written `impl` that
        // references the broken component, and each call. A transformed diagnostic whose signature
        // the ledger has already recorded is such a re-report, so it is suppressed and only the
        // first occurrence is shown; the key scheme lives with the [`DedupLedger`]. Only the tool's
        // own transformed diagnostics are de-duplicated here; an untouched `rustc` error is merged
        // only at the flush, when rustc reports it twice on one line ([`merge_repeats`]). cargo
        // re-counts the diagnostics the emitter produces, so a suppressed re-report drops out of
        // its "N errors" summary as well, keeping the count consistent. A resolved failure records
        // its class headlines, not the cause it displays, since a declined re-report of the same
        // failure is worded with a class sentence.
        let headers: Vec<&str> = if class_headers.is_empty() {
            main_message_text(&diag).into_iter().collect()
        } else {
            class_headers.iter().map(String::as_str).collect()
        };
        if rewritten
            && self.dedup.check_and_record(
                cause_sig.as_deref(),
                || message_signature(&diag),
                &headers,
            )
        {
            return;
        }
        // Buffer rather than emit now: a resolution has its `root cause:` note rendered at flush,
        // where the emission order lets it elide what an earlier block drew, and a consumer failure
        // is additionally grouped there with every failure sharing one of its root causes. Everything
        // else is emitted verbatim in place. The flush happens in `Drop`, the only point after every
        // diagnostic has arrived.
        match resolution {
            Some((resolved, note, coalescing)) => {
                self.push(BufEntry::Resolved(Box::new(ResolvedEntry {
                    coalescible: coalescing,
                    resolved,
                    note,
                    diag,
                    original: original.clone(),
                })));
            }
            None => self.push(BufEntry::Plain(Box::new(diag))),
        }
    }
}

impl<E: Emitter> Emitter for CgpEmitter<E> {
    /// Transform and buffer `diag`, containing a panic in the transform: the diagnostic is then
    /// buffered as rustc built it, with a note saying why, and the compilation goes on.
    fn emit_diagnostic(&mut self, diag: DiagInner) {
        if let Some(original) =
            transform_or_original(diag, |diag, original| self.transform(diag, original))
        {
            self.push(BufEntry::Plain(Box::new(original)));
        }
    }

    fn source_map(&self) -> Option<&SourceMap> {
        self.source_map.as_deref()
    }

    fn emit_artifact_notification(&mut self, path: &Path, artifact_type: &str) {
        self.output()
            .inner
            .emit_artifact_notification(path, artifact_type);
    }

    fn emit_timing_section(&mut self, record: TimingRecord, event: TimingEvent) {
        self.output().inner.emit_timing_section(record, event);
    }

    fn emit_future_breakage_report(&mut self, diags: Vec<DiagInner>) {
        self.output().inner.emit_future_breakage_report(diags);
    }

    fn emit_unused_externs(&mut self, lint_level: rustc_lint_defs::Level, unused_externs: &[&str]) {
        self.output()
            .inner
            .emit_unused_externs(lint_level, unused_externs);
    }

    fn should_show_explain(&self) -> bool {
        self.output().inner.should_show_explain()
    }

    fn supports_color(&self) -> bool {
        self.output().inner.supports_color()
    }
}

impl<E: Emitter> Drop for CgpEmitter<E> {
    /// Flush the buffered diagnostics when the compiler drops the emitter. This runs during the
    /// `DiagCtxt`'s own teardown, *after* every diagnostic has been handed to `emit_diagnostic` but
    /// while the inner emitter (a field of `self`, dropped only after this returns) is still alive —
    /// the one place a "list every affected consumer" block can be built, since the `Emitter` trait
    /// offers no end-of-compilation hook. Diagnostics were counted by the `DiagCtxt` as they
    /// arrived, so deferring their *rendering* to here does not change the error count.
    ///
    /// The rendering is contained: if it panics, every buffered diagnostic is emitted as rustc built
    /// it, a resolution carrying a note saying why. This runs while the thread unwinds from rustc's
    /// fatal-error abort, which is how every failing compilation ends, and a panic contained there
    /// does not escape the destructor, so it does not abort the process.
    fn drop(&mut self) {
        let buffer = std::mem::take(&mut self.output().buffer);
        if buffer.is_empty() {
            return;
        }
        let to_emit = render_or_untransformed(
            buffer,
            |buffer| {
                // Only a resolution's note and a merged block are post-processed here, so the
                // ambiguous names are read only when the buffer holds a resolution.
                let ambiguous = if buffer
                    .iter()
                    .any(|entry| matches!(entry, BufEntry::Resolved(_)))
                {
                    self.facts.ambiguous_names(Origin::Rewritten)
                } else {
                    &EMPTY_AMBIGUOUS_NAMES
                };
                let repeats = if self.facts.cgp_used.get() == Some(&true) {
                    self.source_map.as_deref()
                } else {
                    None
                };
                render(buffer, ambiguous, repeats)
            },
            |entry, note| entry.untransformed(Some(note)),
        );
        let mut output = self.output();
        for diag in to_emit {
            output.inner.emit_diagnostic(diag);
        }
    }
}
