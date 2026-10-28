//! The wrapping [`Emitter`] that transforms CGP diagnostics before delegating.

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

use cargo_cgp_error_processing::rewrite::ComponentNameMap;
use cargo_cgp_error_processing::{
    AmbiguousNames, Causes, ChainNode, DedupLedger, Origin, PendingNote, Resolved,
    coalesce_underived_fields, coalesced_headline, explain_footer_codes, fix_help_messages,
    group_by_shared_cause, is_explain_footer_text, postprocess_message,
};
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::timings::TimingRecord;
use rustc_errors::{DiagInner, DiagMessage, Level, Style, Sublevel, Suggestions};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;
use crate::emitter::attribute_splice::drop_attribute_splices;
use crate::emitter::buffer::{BufEntry, ResolvedEntry};
use crate::emitter::edit::{
    bound_candidate_lists, main_message_text, mentions_hasfield_impls, merged_span,
    message_signature, postprocess_messages, postprocess_multispan, subdiag,
};
use crate::emitter::fallback::{render_or_untransformed, transform_or_original};
use crate::emitter::generated_names::drop_generated_names;
use crate::emitter::output::{Output, lock_output, register_output};
use crate::emitter::pipeline::{BufferEdit, Reshape, Verdict, apply_reshape};
use crate::emitter::repeats::merge_repeats;
use crate::emitter::stage::{
    CgpImplMisuseStage, MacrolessProviderStage, Resolution, ResolvedFailure, StructuralStage,
    UnimportedStage, consumer_trait_as_key, is_cgp_impl_sibling_entry, missing_use_provider,
    orphan_conflict, provider_trait_in_uses, question_mark_cascade, resolve_or_rewrite,
    shared_dispatch_method, undeclared_requirement, undeclared_trait, unresolved_wiring_overflow,
    wiring_conflict,
};
use crate::resolve::{self, ResolveCache};

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
    /// Whether the crate being compiled uses CGP ([`resolve::crate_uses_cgp`]), decided on the first
    /// diagnostic for which the compiler can answer. A crate that does not use CGP can carry no CGP
    /// encoding, so its untransformed diagnostics are left exactly as rustc built them.
    cgp_used: OnceCell<bool>,
    /// The type names that name more than one item ([`resolve::ambiguous_type_names`]), decided on
    /// the first diagnostic for which the compiler can answer, so the post-processing keeps the
    /// qualifier that tells such a name's items apart. `ambiguous` knows the glob-imported names
    /// alone; `ambiguous_with_glob_items` also knows the items those globs bring in, read from HIR,
    /// so it serves only a diagnostic the tool rewrote, which arrives after lowering.
    ambiguous: OnceCell<AmbiguousNames>,
    ambiguous_with_glob_items: OnceCell<AmbiguousNames>,
}

/// The subtrees a flush has already drawn, shared across its blocks so a later one `(*)`-elides
/// what an earlier one showed rather than repeating it.
type ChainNodeSet = HashSet<ChainNode>;

/// A rebuilt "detailed explanations" footer note: `template` cloned from the original footer
/// diagnostic (keeping its `FailureNote` level and empty span) with `message` as its only text.
fn footer_note(template: &DiagInner, message: &str) -> DiagInner {
    let mut note = template.clone();
    note.messages = vec![(DiagMessage::Str(message.to_string().into()), Style::NoStyle)];
    note
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
            cgp_used: OnceCell::new(),
            ambiguous: OnceCell::new(),
            ambiguous_with_glob_items: OnceCell::new(),
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

    /// Build the one merged block for a coalesced group of several failures sharing a root cause: a
    /// headline stating the cause once ([`coalesced_headline`]), the first failing site in the source
    /// as its caret and every other site as a labelled secondary span ([`merged_span`]), and
    /// one root-cause note built by folding *every* member's causes into a single dependency graph,
    /// so a chain that runs through another collapses into it, independent chains to the shared
    /// cause render side by side, and no member's chain is dropped. The headline takes the class
    /// code of the member written first in the source.
    fn merged_diag(
        &self,
        resolveds: &[&Resolved],
        diags: &[&DiagInner],
        seen: &mut ChainNodeSet,
    ) -> DiagInner {
        // Every cause across the coalesced consumers. The dependency graph then merges what they
        // share — a consumer whose chain runs through another collapses into it, while independent
        // chains to one cause render side by side — so no chain is dropped and every consumer appears.
        // These members were grouped precisely *because* they share a cause, so each carries its own
        // copy; `Causes::union` folds those copies back into one cause holding every path — the
        // one-cause-per-distinct-leaf invariant the wording below assumes, kept by construction.
        let causes = Causes::union(resolveds.iter().map(|resolved| resolved.causes.clone()));
        let causes = coalesce_underived_fields(&causes);
        let headline = coalesced_headline(resolveds, &causes);
        // The fixes follow the note that explains them, as they do in a block of its own.
        let helps = fix_help_messages(&causes);
        let mut children: Vec<_> = PendingNote {
            causes,
            header_leaf: None,
        }
        .render(seen)
        .into_iter()
        .map(|note| subdiag(Sublevel::Note, note))
        .collect();
        children.extend(helps.into_iter().map(|help| subdiag(Sublevel::Help, help)));

        let mut diag = diags[0].clone();
        diag.messages = vec![(DiagMessage::Str(headline.into()), Style::NoStyle)];
        diag.span = merged_span(diags);
        diag.children = children;
        diag.suggestions = Suggestions::Enabled(Vec::new());
        // The header, notes, and helps are freshly built, so post-process them (resugar `Symbol!`,
        // strip CGP prefixes) as the streaming path does for a rewritten diagnostic.
        self.postprocess(&mut diag, Origin::Rewritten);
        diag
    }

    /// Render the buffered diagnostics for the flush, coalescing each group of coalescible
    /// failures that **share a root cause** into one block at the position of its first member.
    /// Called only from [`Drop`], where the inner emitter is still alive to emit the result.
    fn render(&self, buffer: &[BufEntry]) -> Vec<DiagInner> {
        // The coalescible entries in arrival order, grouped by the root causes they share. Grouping
        // on shared causes rather than on one whole-failure key is what keeps a single mistake in a
        // single block: the depths it surfaces at each see a different *subset* of its causes, so no
        // two of them ever carried equal keys (see [`group_by_shared_cause`]).
        let positions: Vec<usize> = buffer
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                BufEntry::Resolved(entry) if entry.coalescible => Some(index),
                _ => None,
            })
            .collect();
        let resolveds: Vec<&Resolved> = positions
            .iter()
            .map(|&index| match &buffer[index] {
                BufEntry::Resolved(entry) => &entry.resolved,
                _ => unreachable!("indices came from the coalescible entries"),
            })
            .collect();
        let groups: Vec<Vec<usize>> = group_by_shared_cause(&resolveds)
            .into_iter()
            .map(|group| group.into_iter().map(|member| positions[member]).collect())
            .collect();
        let group_of: HashMap<usize, usize> = groups
            .iter()
            .enumerate()
            .flat_map(|(group, members)| members.iter().map(move |&index| (index, group)))
            .collect();

        // The subtrees drawn so far, threaded through every block in emission order so a later one
        // truncates at what an earlier one already showed. This is why notes are rendered here
        // rather than on arrival: only now is that order known.
        let mut seen = ChainNodeSet::new();
        let mut emitted: HashSet<usize> = HashSet::new();
        let mut to_emit: Vec<DiagInner> = Vec::new();
        for (index, entry) in buffer.iter().enumerate() {
            match entry {
                BufEntry::Plain(diag) => to_emit.push((**diag).clone()),
                BufEntry::Resolved(entry) => {
                    let Some(&group) = group_of.get(&index) else {
                        // A resolution that never coalesces: render its own note and emit it here.
                        let mut diag = entry.diag.clone();
                        self.insert_note(&mut diag, &entry.note, &mut seen);
                        to_emit.push(diag);
                        continue;
                    };
                    if !emitted.insert(group) {
                        continue;
                    }
                    let members = &groups[group];
                    if members.len() == 1 {
                        // A group's first member is where it is emitted, so that member is `entry`.
                        let mut diag = entry.diag.clone();
                        self.insert_note(&mut diag, &entry.note, &mut seen);
                        to_emit.push(diag);
                    } else {
                        let mut pairs: Vec<(&Resolved, &DiagInner)> = members
                            .iter()
                            .map(|&index| match &buffer[index] {
                                BufEntry::Resolved(member) => (&member.resolved, &member.diag),
                                _ => unreachable!("indices came from the coalescible entries"),
                            })
                            .collect();
                        // The block's code and first caret come from the member written first in
                        // the source, so the code names the class of the entry the reader sees first.
                        pairs.sort_by_key(|(_, diag)| {
                            diag.span.primary_span().map(|span| span.lo())
                        });
                        let (resolveds, diags): (Vec<&Resolved>, Vec<&DiagInner>) =
                            pairs.into_iter().unzip();
                        to_emit.push(self.merged_diag(&resolveds, &diags, &mut seen));
                    }
                }
            }
        }

        // A diagnostic rustc reported twice is one mistake; merge the repeat, in a crate known to
        // use CGP, so a plain Rust error is never touched.
        if self.cgp_used.get() == Some(&true)
            && let Some(source_map) = self.source_map.as_deref()
        {
            to_emit = merge_repeats(to_emit, source_map);
        }
        // Now that the surviving set is final, make the "detailed explanations" footer agree with it.
        Self::rebuild_explain_footers(&mut to_emit);
        to_emit
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

    /// Post-process a diagnostic after transforming it — the final cleanup pass, over every
    /// message and span label of the diagnostic and its children. It strips module paths,
    /// resugars `Symbol!` and `Path!` and the type-level lists, and rewords an unmet `HasField`
    /// bound. Whether the context implements `HasField` for any field is a fact of the whole
    /// diagnostic (the "similar impl" landmark can sit far from the clause), so it is decided once
    /// up front and passed into each per-message rewrite.
    ///
    /// `origin` says whether the tool rewrote the diagnostic. An untransformed one changes only
    /// where the chain recovers a CGP encoding (see [`Origin`]), and only in a crate that loads
    /// CGP, so a plain Rust error passes through exactly as rustc built it.
    fn postprocess(&self, diag: &mut DiagInner, origin: Origin) {
        if origin == Origin::Untransformed && !self.crate_uses_cgp() {
            return;
        }
        let has_field_impls = mentions_hasfield_impls(diag);
        let ambiguous = self.ambiguous_names(origin);
        postprocess_messages(&mut diag.messages, has_field_impls, origin, ambiguous);
        postprocess_multispan(&mut diag.span, has_field_impls, origin, ambiguous);
        for child in &mut diag.children {
            postprocess_messages(&mut child.messages, has_field_impls, origin, ambiguous);
            postprocess_multispan(&mut child.span, has_field_impls, origin, ambiguous);
        }
    }

    /// The type names that name more than one item, decided once the compiler can answer. Before
    /// then the set is empty, which strips every qualifier. A message the tool rewrote also knows
    /// which item each glob import brings in. Reading that needs the HIR, which an untransformed
    /// diagnostic emitted during name resolution would force the compiler to build under the
    /// emitter, so an untransformed message gets the glob-imported names alone.
    fn ambiguous_names(&self, origin: Origin) -> &AmbiguousNames {
        static EMPTY: LazyLock<AmbiguousNames> = LazyLock::new(AmbiguousNames::default);
        let read_glob_items = origin == Origin::Rewritten;
        let cell = if read_glob_items {
            &self.ambiguous_with_glob_items
        } else {
            &self.ambiguous
        };
        if let Some(names) = cell.get() {
            return names;
        }
        match rustc_middle::ty::tls::with_opt(|tcx| {
            tcx.and_then(|tcx| resolve::ambiguous_type_names(tcx, read_glob_items))
        }) {
            Some(names) => cell.get_or_init(|| names),
            None => &EMPTY,
        }
    }

    /// Whether the crate being compiled uses CGP, decided once the compiler can answer. Before then
    /// (a parse or name-resolution error) the answer is `false`, which leaves the diagnostic as
    /// rustc built it.
    fn crate_uses_cgp(&self) -> bool {
        if let Some(&known) = self.cgp_used.get() {
            return known;
        }
        let Some(known) =
            rustc_middle::ty::tls::with_opt(|tcx| tcx.and_then(resolve::crate_uses_cgp))
        else {
            return false;
        };
        let _ = self.cgp_used.set(known);
        known
    }

    /// Rewrite the trailing "detailed explanations" footer of a flushed diagnostic list so it names
    /// `rustc --explain` codes only for errors still shown.
    ///
    /// rustc builds that footer in `print_error_count` from every error code it *registered* as
    /// diagnostics arrived, which is a superset of what this emitter emits: it suppresses a
    /// re-reported failure, a `?`-operator cascade, and a `#[cgp_impl]` mistake's sibling burst, and
    /// it *merges* the failures sharing a root cause into one block whose code is the first member's.
    /// Left alone the footer contradicts the output, offering `--explain` for a code no error carries.
    ///
    /// It runs over the flushed list rather than on the footer's arrival because only here is the
    /// surviving set final — coalescing happens at flush, after every diagnostic has been buffered.
    /// The kept codes are the footer's own codes filtered by what survived, never a fresh set built
    /// from the survivors: rustc lists only codes that *have* an extended explanation, and
    /// intersecting preserves that filtering instead of second-guessing it. A footer whose codes all
    /// survive is rewritten to the identical text, so output the emitter did not touch is unaffected.
    fn rebuild_explain_footers(to_emit: &mut Vec<DiagInner>) {
        let is_footer = |diag: &DiagInner| {
            diag.level() == Level::FailureNote
                && main_message_text(diag).is_some_and(is_explain_footer_text)
        };
        if !to_emit.iter().any(is_footer) {
            return;
        }

        // The rust codes still shown, and the explainable ones the footer itself names (the list line
        // carries every one, the pointer line only the first, so their union is the whole set).
        let surviving: HashSet<String> = to_emit
            .iter()
            .filter(|diag| diag.level() != Level::FailureNote)
            .filter_map(|diag| diag.code)
            .map(|code| code.to_string())
            .collect();
        let mut explainable: Vec<String> = Vec::new();
        for diag in to_emit.iter().filter(|diag| is_footer(diag)) {
            for code in explain_footer_codes(main_message_text(diag).unwrap_or_default()) {
                if !explainable.contains(&code) {
                    explainable.push(code);
                }
            }
        }
        // A footer line always names at least one code, so parsing none from any of them means the
        // parse failed — rustc reworded — not that the line names nothing. Rebuilding on that would
        // delete the footer outright, so leave it exactly as rustc wrote it: a stale footer is a far
        // smaller fault than a silently missing one, and it is the pre-rebuild behaviour.
        if explainable.is_empty() {
            return;
        }
        let kept: Vec<String> = explainable
            .into_iter()
            .filter(|code| surviving.contains(code))
            .collect();

        let rebuilt: Vec<DiagInner> = to_emit
            .drain(..)
            .filter_map(|diag| {
                if !is_footer(&diag) {
                    return Some(diag);
                }
                let text = main_message_text(&diag).unwrap_or_default();
                let Some(first) = kept.first() else {
                    // Every explainable code was suppressed, so there is nothing left to point at
                    // and both footer lines go.
                    return None;
                };
                if text.starts_with("Some errors have detailed explanations:") {
                    // The list line is only right for two or more codes; with one, the pointer line
                    // below carries it alone.
                    if kept.len() < 2 {
                        return None;
                    }
                    return Some(footer_note(
                        &diag,
                        &format!(
                            "Some errors have detailed explanations: {}.",
                            kept.join(", ")
                        ),
                    ));
                }
                let message = if kept.len() == 1 {
                    format!("For more information about this error, try `rustc --explain {first}`.")
                } else {
                    format!("For more information about an error, try `rustc --explain {first}`.")
                };
                Some(footer_note(&diag, &message))
            })
            .collect();
        *to_emit = rebuilt;
    }

    /// Render a deferred [`PendingNote`] against the `seen` set shared by this flush and add it to
    /// `diag`, ahead of its `help`s, so each fix follows the cause it repairs. The note is
    /// post-processed on its own, since the rest of the diagnostic was already processed when it
    /// arrived and re-running the chain over it could compound.
    fn insert_note(&self, diag: &mut DiagInner, note: &PendingNote, seen: &mut ChainNodeSet) {
        // A resolver-built note never carries the `` `HasField<…>` is not implemented `` clause the
        // missing-field reword keys on (see
        // cgp-knowledge-base/cargo-cgp/implementation/error-processing.md), so that branch has
        // nothing to match and the flag it needs is moot here.
        const HAS_FIELD_IMPLS: bool = false;
        let notes: Vec<_> = note
            .render(seen)
            .into_iter()
            .map(|text| {
                let text = postprocess_message(
                    &text,
                    HAS_FIELD_IMPLS,
                    Origin::Rewritten,
                    self.ambiguous_names(Origin::Rewritten),
                )
                .unwrap_or(text);
                subdiag(Sublevel::Note, text)
            })
            .collect();
        let first_help = diag
            .children
            .iter()
            .position(|child| child.level == Sublevel::Help)
            .unwrap_or(diag.children.len());
        diag.children.splice(first_help..first_help, notes);
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
            .or_else(|| unresolved_wiring_overflow(&diag))
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
        let resolution = resolve_or_rewrite(
            tcx,
            &mut diag,
            &self.names,
            &self.resolve_cache,
            &self.cgp_impl_misuse,
        );
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
        if self.crate_uses_cgp() {
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
            |buffer| self.render(buffer),
            |entry, note| entry.untransformed(Some(note)),
        );
        let mut output = self.output();
        for diag in to_emit {
            output.inner.emit_diagnostic(diag);
        }
    }
}
