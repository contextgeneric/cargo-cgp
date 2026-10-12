//! The wrapping [`Emitter`] that transforms CGP diagnostics before delegating.

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, rewrite_message, rewrite_required_for, rewrite_wiring_overflow,
};
use cargo_cgp_error_processing::{
    AmbiguousNames, Causes, CgpImplMisuse, ChainNode, DedupLedger, DiagKind, Leaf,
    MissingUseProvider, Origin, OrphanConflict, PendingNote, Resolved, StructuralMistake,
    UndeclaredTrait, WiringOverflow, cause_signature, cgp_impl_misuse_help,
    coalesce_underived_fields, coalesced_headline, explain_footer_codes, fix_help_messages,
    group_by_shared_cause, internal_error_note, is_explain_footer_text, is_method_bounds_text,
    is_recursion_limit_help_text, is_unbounded_type_param_item_text, mentions_orphan_param_text,
    missing_use_provider_help, orphan_conflict_help, plan_cgp_impl_misuse,
    plan_missing_use_provider, plan_orphan_conflict, plan_resolved, plan_structural_mistake,
    plan_undeclared_trait, plan_wiring_conflict, postprocess_message, structural_mistake_help,
    undeclared_trait_help, wiring_conflict_help, wiring_overflow_help,
};
use rustc_errors::codes::{
    E0107, E0117, E0119, E0186, E0207, E0210, E0271, E0275, E0277, E0308, E0425, E0428, E0599,
};
use rustc_errors::emitter::{Emitter, TimingEvent};
use rustc_errors::timings::TimingRecord;
use rustc_errors::{
    DiagArgValue, DiagInner, DiagMessage, Level, MultiSpan, Style, Sublevel, Suggestions,
};
use rustc_span::Span;
use rustc_span::source_map::SourceMap;

use crate::component_map::build_name_map_from_tls;
use crate::config::BUG_REPORT_URL;
use crate::emitter::buffer::{BufEntry, ResolvedEntry};
use crate::emitter::containment::contain;
use crate::emitter::edit::{
    bound_candidate_lists, check_table_span, diag_kind, diagnostic_spans, is_question_mark_cascade,
    main_message_text, mentions_hasfield_impls, mentions_wiring, merged_span, message_signature,
    postprocess_messages, postprocess_multispan, replace_header, rewrite_messages,
    strip_method_probe_advice, subdiag,
};
use crate::emitter::output::{Output, lock_output, register_output};
use crate::emitter::test_panic::{TestPanic, inject};
use crate::resolve::{self, ConflictAction, ConflictTrait, DetectedCgpImplMisuse, ResolveCache};

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
    /// compilation. Used to drop a downstream `?`-operator cascade
    /// ([`is_question_mark_cascade`]) that lands on the same expression: once a wiring bound on
    /// `expr` fails, the type of `expr?` cannot be resolved, so rustc adds a `Try`/`FromResidual`
    /// error at the same span that restates the wiring failure and dumps the unresolved projected
    /// type — noise over the CGP error already shown. rustc emits the wiring failure before its
    /// cascade, so the span is recorded by the time the cascade arrives.
    cgp_spans: Vec<Span>,
    /// The structural mistakes already reported, so a mistake rustc reports as several errors
    /// becomes one.
    structural_reported: Vec<StructuralMistake>,
    /// The `#[cgp_impl]` header-trait mistakes in the crate — a consumer trait or a non-CGP trait
    /// named where the provider trait belongs — detected once from the compiler and reused. `None`
    /// until first computed under an available `TyCtxt` (the macro-lowering errors this recognizes
    /// arrive before trait solving, so the first candidate may precede one); an empty `Vec` means
    /// "computed, none found". Owned data (names plus `Copy` spans), so it outlives the `TyCtxt`.
    cgp_impl_misuses: Option<Vec<DetectedCgpImplMisuse>>,
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
    /// The stage made to panic on purpose, set only by the UI fixtures that pin the panic fallback
    /// (see [`TestPanic`]).
    test_panic: Option<TestPanic>,
}

/// The subtrees a flush has already drawn, shared across its blocks so a later one `(*)`-elides
/// what an earlier one showed rather than repeating it.
type ChainNodeSet = HashSet<ChainNode>;

/// What [`CgpEmitter::transform_resolved`] leaves for the rest of the transform: the note the
/// flush renders, the class headline the de-duplication ledger records, and whether the block may
/// coalesce with others sharing its cause.
struct TransformedResolution {
    note: PendingNote,
    /// The class headline, and the provider form a declined re-report may carry instead.
    headers: Vec<String>,
    coalesces: bool,
}

/// The outcome of recognizing a diagnostic as part of the cascade a `#[cgp_impl]` header-trait
/// mistake produces.
enum CgpImplMisuseAction {
    /// Reshape the `E0107` into the coded header for `misuse`; the two spans scope the sibling purge.
    Reshape {
        misuse: CgpImplMisuse,
        impl_span: Span,
        macro_span: Span,
    },
    /// Drop this diagnostic as a redundant consequence of the mistake.
    Suppress,
}

/// Whether `diag` is a sibling macro-lowering error of a `#[cgp_impl]` mistake — landing in the impl
/// body (`impl_span`) or at the macro call-site the synthesized tokens share (`macro_span`, where
/// `E0425`/`E0207` sit, outside the impl body). `E0425`/`E0186`/`E0207` are always siblings when they
/// land there; an `E0308` type-mismatch cascade (from a malformed inner-provider bound) is a sibling
/// only when it mentions the generated `__Context__`, so a genuine user type error is never dropped.
fn is_cgp_impl_sibling(diag: &DiagInner, impl_span: Span, macro_span: Span) -> bool {
    let Some(primary) = diag.span.primary_span() else {
        return false;
    };
    if !(impl_span.overlaps(primary) || macro_span.overlaps(primary)) {
        return false;
    }
    match diag.code {
        Some(E0425) | Some(E0186) | Some(E0207) => true,
        Some(E0308) => mentions_generated_context(diag),
        _ => false,
    }
}

/// Whether any plain-string message of `diag` — its main message, a child's, or a span label —
/// satisfies `pred`. Used to recognize a diagnostic whose *main* message is a Fluent (non-`Str`)
/// message but whose help or label carries the signal as plain text.
fn diag_mentions(diag: &DiagInner, pred: fn(&str) -> bool) -> bool {
    let in_messages = |messages: &[(DiagMessage, Style)]| {
        messages
            .iter()
            .any(|(message, _)| matches!(message, DiagMessage::Str(text) if pred(text)))
    };
    let in_labels = |span: &MultiSpan| {
        span.span_labels()
            .into_iter()
            .any(|label| matches!(label.label, Some(DiagMessage::Str(text)) if pred(&text)))
    };
    in_messages(&diag.messages)
        || in_labels(&diag.span)
        || diag
            .children
            .iter()
            .any(|child| in_messages(&child.messages) || in_labels(&child.span))
}

/// Whether any message of `diag` or its children names the generated `__Context__` parameter — the
/// tell that an `E0308` mismatch is a macro-lowering cascade rather than a user's own type error.
fn mentions_generated_context(diag: &DiagInner) -> bool {
    let in_messages = |messages: &[(DiagMessage, Style)]| {
        messages.iter().any(|(message, _)| {
            matches!(message, DiagMessage::Str(text) if text.contains("__Context__"))
        })
    };
    in_messages(&diag.messages)
        || diag
            .children
            .iter()
            .any(|child| in_messages(&child.messages))
}

/// Whether a buffered entry is a sibling of the impl at `(impl_span, macro_span)` — the shape purged
/// when the reshaped `E0107` is produced, since such a sibling (notably the name-resolution `E0425`)
/// can arrive before the `E0107` that recognizes the mistake.
fn is_cgp_impl_sibling_entry(entry: &BufEntry, impl_span: Span, macro_span: Span) -> bool {
    matches!(entry, BufEntry::Plain(diag) if is_cgp_impl_sibling(diag, impl_span, macro_span))
}

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
    pub fn new(inner: E, source_map: Option<Arc<SourceMap>>, test_panic: Option<TestPanic>) -> Self
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
            structural_reported: Vec::new(),
            cgp_impl_misuses: None,
            cgp_used: OnceCell::new(),
            ambiguous: OnceCell::new(),
            ambiguous_with_glob_items: OnceCell::new(),
            test_panic,
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

        // Now that the surviving set is final, make the "detailed explanations" footer agree with it.
        Self::rebuild_explain_footers(&mut to_emit);
        to_emit
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

    /// Why the `E0275` overflow `diag` never resolves, when walking its check entry's wiring shows
    /// it: a cycle, or a lookup that grows ([`resolve::overflow_walk`]). `None` for any other
    /// diagnostic, and for an overflow the walk does not explain (one that follows from an
    /// unresolved type, say), which keeps rustc's wording.
    fn wiring_overflow(&self, diag: &DiagInner) -> Option<WiringOverflow> {
        if diag.code != Some(E0275) {
            return None;
        }
        let span = diag.span.primary_span()?;
        rustc_middle::ty::tls::with_opt(|tcx| {
            match resolve::overflow_walk(tcx?, &self.resolve_cache, span) {
                resolve::OverflowWalk::Cut(reason) => Some(reason),
                resolve::OverflowWalk::NotCut | resolve::OverflowWalk::NoEntry => None,
            }
        })
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

    /// Recognize `diag` as a structural mistake in a wiring or its checks, recovered from the
    /// generated items at its caret: an `E0119` between two identical check impls is a check table
    /// listing one entry twice, an `E0428` on a check trait is two tables whose check traits share a
    /// name, and an `E0207` is a `for` loop whose key does not use its variable or a
    /// `#[default_impl]` on a generic provider impl. `None` for any other diagnostic.
    fn structural_mistake(&self, diag: &DiagInner) -> Option<StructuralMistake> {
        let primary_span = diag.span.primary_span()?;
        match diag.code {
            Some(E0119) => rustc_middle::ty::tls::with_opt(|tcx| {
                resolve::detect_duplicate_check(tcx?, primary_span)
            }),
            // Name resolution reports a name defined twice, and the derived diagnostic carries the
            // name as an argument rather than in a plain-string message.
            Some(E0428) => match diag.args.get("name") {
                Some(DiagArgValue::Str(name)) => {
                    resolve::detect_duplicate_check_trait(name, primary_span)
                }
                _ => None,
            },
            Some(E0207) => rustc_middle::ty::tls::with_opt(|tcx| {
                let tcx = tcx?;
                resolve::detect_loop_key_without_variable(tcx, primary_span)
                    .or_else(|| resolve::detect_generic_default_impl(tcx, primary_span))
            }),
            _ => None,
        }
    }

    /// Recognize `diag` as an undeclared-trait failure — a CGP trait called in a
    /// `#[cgp_fn]`/`#[cgp_impl]` body without being declared via `#[uses(…)]`, so its method cannot
    /// resolve on the generated `__Context__` generic. Returns the trait to declare, or `None`
    /// when it is not one. Gated to the method-bounds `E0599` shape, then confirmed structurally by
    /// [`resolve::detect_undeclared_trait`] (a generated blanket impl with a bare-parameter
    /// `Self`, a CGP trait method call, and no matching `where` bound).
    fn undeclared_trait(&self, diag: &DiagInner) -> Option<UndeclaredTrait> {
        if diag.code != Some(E0599) || !main_message_text(diag).is_some_and(is_method_bounds_text) {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        let spans = diagnostic_spans(diag);
        rustc_middle::ty::tls::with_opt(|tcx| {
            resolve::detect_undeclared_trait(tcx?, primary_span, &spans)
        })
    }

    /// Recognize a missing-`#[use_provider]` failure — a higher-order provider calling an inner
    /// provider (`Inner::method(self)`) it never imported, so the inner parameter is unbounded and
    /// the associated-function call cannot resolve. Returns the inner provider and the provider trait
    /// to import it as, or `None`. Gated to the "no associated item for a type parameter" `E0599`
    /// shape — reported during typeck of the calling body, so the detector's queries are cached and
    /// safe (unlike the resolution-class `E0599` emitted mid-`clauses_of`) — then confirmed
    /// structurally by [`resolve::detect_missing_use_provider`].
    fn missing_use_provider(&self, diag: &DiagInner) -> Option<MissingUseProvider> {
        // The `E0599`'s main message is a Fluent (non-`Str`) message, so the shape is recognized by
        // its plain-string help/label instead.
        if diag.code != Some(E0599) || !diag_mentions(diag, is_unbounded_type_param_item_text) {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        rustc_middle::ty::tls::with_opt(|tcx| {
            resolve::detect_missing_use_provider(tcx?, primary_span)
        })
    }

    /// Detect the crate's `#[cgp_impl]` header-trait mistakes once, memoizing the result. Runs only
    /// when a `TyCtxt` is in scope; the macro-lowering errors that trigger detection can be emitted
    /// before trait solving puts one there, so a first attempt may find none and detection is
    /// retried on a later diagnostic (the reshaped `E0107`, in the type-lowering phase, always has
    /// one).
    fn ensure_cgp_impl_misuses(&mut self) {
        if self.cgp_impl_misuses.is_some() {
            return;
        }
        if let Some(detected) =
            rustc_middle::ty::tls::with_opt(|tcx| tcx.map(resolve::detect_cgp_impl_misuses))
        {
            self.cgp_impl_misuses = Some(detected);
        }
    }

    /// The action for a diagnostic in the burst a `#[cgp_impl]` header mistake produces: reshape the
    /// `E0107` whose caret sits on the misused trait name into the coded header, or suppress a
    /// sibling macro-lowering error (`E0425`/`E0186`/`E0207`) landing inside the same impl as a
    /// redundant consequence. `None` for anything unrelated.
    fn cgp_impl_misuse_action(&self, diag: &DiagInner) -> Option<CgpImplMisuseAction> {
        let misuses = self.cgp_impl_misuses.as_deref()?;
        if misuses.is_empty() {
            return None;
        }
        let primary = diag.span.primary_span()?;
        if diag.code == Some(E0107)
            && let Some(misuse) = misuses
                .iter()
                .find(|misuse| misuse.trait_ref_span.overlaps(primary))
        {
            return Some(CgpImplMisuseAction::Reshape {
                misuse: misuse.misuse.clone(),
                impl_span: misuse.impl_span,
                macro_span: misuse.macro_span,
            });
        }
        if misuses
            .iter()
            .any(|misuse| is_cgp_impl_sibling(diag, misuse.impl_span, misuse.macro_span))
        {
            return Some(CgpImplMisuseAction::Suppress);
        }
        None
    }

    /// Whether `resolved` is the downstream check re-report of a detected *consumer-trait*
    /// `#[cgp_impl]` mistake — every cause a `NotAProvider` leaf naming that mistake's provider
    /// struct and the provider trait its header should have targeted. Such a failure is a pure
    /// consequence of the mistake already reported as `[CGP-E032]` (the provider struct cannot be a
    /// provider because its impl targets the wrong trait), so it is suppressed rather than shown as
    /// its own `[CGP-E115]` block. A requirement of that impl which fails on its own, such as a field
    /// the context lacks, is a separate mistake: the walk resolves it to its own cause through the
    /// impl as written, so a report carrying it is kept.
    fn is_cgp_impl_misuse_check_report(&self, resolved: &Resolved) -> bool {
        let Some(misuses) = self.cgp_impl_misuses.as_deref() else {
            return false;
        };
        if misuses.is_empty() || resolved.causes.is_empty() {
            return false;
        }
        resolved.causes.iter().all(|cause| {
            matches!(
                &cause.leaf,
                Leaf::NotAProvider { provider, provider_trait }
                    if misuses.iter().any(|misuse| matches!(
                        &misuse.misuse,
                        CgpImplMisuse::ConsumerTrait { provider: expected, .. }
                            if provider == &misuse.self_ty && provider_trait == expected
                    ))
            )
        })
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

    /// Resolve `diag`'s failure to its root-cause dependency tree(s), or `None` when the resolver
    /// cannot trace it to a CGP component failure (so the caller falls back to the in-place text
    /// rewrite). A candidate is every diagnostic that reports an unmet obligation (`E0271`,
    /// `E0275`, `E0277`, and a *method-bounds* `E0599`), whether or not it names a CGP trait —
    /// because a failure *not* worded in CGP terms can still be a consequence of a CGP component
    /// failing (a manual `impl` that forwards to a wired method, a downstream trait bound that needs
    /// it), and [`resolve`] traces the dependency chain to find out. It yields `None` for everything whose chain does not reach a CGP cause. Returns the
    /// primary span alongside the resolution so the caret can be re-aimed at the entry.
    ///
    /// The `E0599` arm is narrowed to the "the method `…` exists … but its trait bounds were not
    /// satisfied" shape — the consumer-method call the use-site anchor handles. A *resolution*-class
    /// `E0599` (`no variant named …`, `no associated item …`) is not a wiring failure and, worse, is
    /// emitted *during* type lowering / `clauses_of`, while that query is mid-flight: running the
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
        // Only a diagnostic reporting an unmet obligation has a chain to walk: a projection or
        // trait-bound failure, an overflow evaluating a requirement, or a method whose bounds fail.
        // A coherence conflict (`E0119`) names CGP traits too, but two impls overlapping is not a
        // bound left unmet, so walking it would invent a missing-impl cause.
        if !matches!(diag.code, Some(E0271) | Some(E0275) | Some(E0277)) && !e0599_method_bounds {
            return None;
        }
        // A `?`-operator error is about `Try` on the call's result, not about the call's bounds; the
        // cascade gate in `emit_diagnostic` drops one that follows a CGP failure, and any other is
        // left as rustc wrote it rather than re-anchored onto the call it overlaps.
        if is_question_mark_cascade(diag) {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        let cache = &self.resolve_cache;
        let (resolved, at_call) = rustc_middle::ty::tls::with_opt(|tcx| {
            let tcx = tcx?;
            let spans = diagnostic_spans(diag);
            // The impl-site, wrapper-chain, and call-site anchors re-anchor a diagnostic onto code it
            // sits inside, so they match only where rustc aimed it: a secondary label or a note can
            // point at any code the failure passed through.
            let primary = diag.span.primary_spans();
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
                // A `#[check_providers]` entry is anchored the same way, on the provider it lists.
                .or_else(|| resolve::resolve_provider_check(tcx, cache, primary_span))
                .or_else(|| resolve::resolve_impl_site(tcx, cache, primary))
                .or_else(|| resolve::resolve_wrapper_chain(tcx, cache, primary))
                .or_else(|| resolve::resolve_use_site(tcx, cache, &spans))
                // A namespace-joined context's wiring lives in the namespace, not its own
                // `DelegateComponent` impls, so the per-component re-check above finds nothing;
                // anchoring on the consumer trait the diagnostic names and walking through the
                // namespace recovers it.
                .or_else(|| resolve::resolve_use_site_consumer(tcx, cache, &spans));
            if let Some(resolved) = resolved {
                return Some((resolved, false));
            }
            // The next resort re-reads the failing *call expression* itself — the anchor for a
            // consumer-method `E0277` whose spans never touch the context's definition (a
            // `Code`-dispatched handler pipeline that matches unconditionally), and for a direct
            // call to a `#[cgp_fn]` trait method. A resolution from here is flagged, so the
            // header is worded from the trait the call needs rather than from whichever provider
            // bound rustc's headline stopped on.
            if let Some(resolved) = resolve::resolve_call_site(tcx, cache, primary) {
                return Some((resolved, true));
            }
            // Last: a `#[cgp_fn]` / `#[blanket_trait]` blanket trait the diagnostic names in its
            // spans, required through a `where` bound or supertrait rather than a direct call. This
            // is gated to `E0277` — a trait *used as a bound* — deliberately: an `E0599` method
            // call is the call-site anchor's domain, and a *generic consumer* method call whose deep
            // trait bound is a note (not the failure the diagnostic is about) must stay declined
            // when its dispatch parameter is unrecoverable, rather than latch onto that transitive
            // trait (see `generic_consumer_unwritten_arg`).
            if diag.code == Some(E0277)
                && let Some(resolved) = resolve::resolve_use_site_blanket_trait(tcx, cache, &spans)
            {
                return Some((resolved, false));
            }
            None
        })?;
        Some((resolved, primary_span, at_call))
    }

    /// Transform a resolved wiring failure in place from the rustc-free [`plan_resolved`]: replace
    /// the main message when the plan carries a coded header (re-aiming the caret at the failing
    /// entry), then replace the sub-messages with the plan's fix `help`s, dropping rustc's own
    /// suggestions; the `root cause:` note is returned for the flush to render. A resolution
    /// anchored at the call expression (`at_call`) plans as a use-site failure whatever its rustc
    /// code, so its class is the consumer the call needs, except a genuine field-type mismatch,
    /// whose `E0271` class is the more specific `[CGP-E004]`.
    fn transform_resolved(
        &self,
        diag: &mut DiagInner,
        resolved: &Resolved,
        span: Span,
        at_call: bool,
    ) -> TransformedResolution {
        let kind = match diag_kind(diag) {
            kind if at_call && kind != DiagKind::TypeMismatch => DiagKind::MethodNotFound,
            kind => kind,
        };
        let plan = plan_resolved(kind, main_message_text(diag), resolved, &self.names);
        let coalesces = plan.coalesces();

        if let Some(header) = plan.header {
            diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
            // Re-aim the caret at the failing entry alone: the original span labels restate the
            // replaced message, so they no longer apply.
            diag.span = MultiSpan::from_span(span);
        }

        // The `help`s are applied now; the `root cause:` note is not, because rendering it needs the
        // emission order the flush establishes (see [`PendingNote`]). The flush puts it ahead of
        // these `help`s, so a fix follows the cause it repairs.
        diag.children = plan
            .helps
            .into_iter()
            .map(|help| subdiag(Sublevel::Help, help))
            .collect();
        // Drop rustc's structured suggestions along with its notes — for a use-site failure
        // that includes the misleading "use associated function syntax instead".
        diag.suggestions = Suggestions::Enabled(vec![]);

        TransformedResolution {
            note: plan.note,
            headers: plan
                .class_header
                .into_iter()
                .chain(plan.declined_header)
                .collect(),
            coalesces,
        }
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
    fn transform(&mut self, mut diag: DiagInner, original: &DiagInner) {
        // The injected `--verbose` lists every implementor in a failed bound's help; cut the list
        // back to rustc's own length before anything else reads the diagnostic.
        bound_candidate_lists(&mut diag);
        // A `#[cgp_impl]` header naming the wrong trait — the component's consumer trait, or a
        // non-CGP trait — makes the macro generate an inside-out impl of the wrong trait, producing
        // a burst of cryptic macro-lowering errors (E0425/E0107/E0186/E0207) plus a downstream check
        // failure, none naming the mistake. Detect it structurally (once) and reshape the E0107
        // whose caret is on the misused trait name into a `[CGP-E032]`/`[CGP-E033]` header with the
        // fix, suppressing the rest of the cascade — the sibling errors here, the check re-report
        // below.
        if matches!(
            diag.code,
            Some(E0107) | Some(E0425) | Some(E0186) | Some(E0207) | Some(E0308)
        ) {
            // Detection forces HIR/trait-graph queries, which re-enter the `DiagCtxt` lock and abort
            // the compiler if run while an *early-phase* diagnostic is emitting (name resolution,
            // where `E0425` lands, is mid-`hir_owner`). So detection is triggered only by the
            // `E0107` — a type-lowering-phase error, always present for this mistake and safe to
            // query from. The `E0425`/`E0186`/`E0207` siblings that precede it are purged from the
            // buffer at reshape time; those that follow are suppressed inline once the memo is set.
            if diag.code == Some(E0107) {
                self.ensure_cgp_impl_misuses();
            }
            match self.cgp_impl_misuse_action(&diag) {
                Some(CgpImplMisuseAction::Suppress) => return,
                Some(CgpImplMisuseAction::Reshape {
                    misuse,
                    impl_span,
                    macro_span,
                }) => {
                    let primary = diag.span.primary_span();
                    replace_header(&mut diag, plan_cgp_impl_misuse(&misuse));
                    // Keep rustc's caret — already on the misused trait name — but drop its "expected
                    // N generic arguments" label, which no longer fits the reshaped message.
                    if let Some(primary) = primary {
                        diag.span = MultiSpan::from_span(primary);
                    }
                    diag.children = vec![subdiag(Sublevel::Help, cgp_impl_misuse_help(&misuse))];
                    diag.suggestions = Suggestions::Enabled(Vec::new());
                    self.postprocess(&mut diag, Origin::Rewritten);
                    // Drop any sibling of this impl already buffered: E0425 (name resolution)
                    // arrives before this E0107 (type lowering), possibly before a `TyCtxt` was in
                    // scope to detect the mistake, so it was buffered verbatim.
                    self.output()
                        .buffer
                        .retain(|entry| !is_cgp_impl_sibling_entry(entry, impl_span, macro_span));
                    self.record_cgp_spans(&diag);
                    self.push(BufEntry::Plain(Box::new(diag)));
                    return;
                }
                None => {}
            }
        }
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
                        diag.children.push(subdiag(Sublevel::Help, help));
                    }
                    // A rewritten diagnostic: bare `@…` paths.
                    self.postprocess(&mut diag, Origin::Rewritten);
                    self.record_cgp_spans(&diag);
                    self.push(BufEntry::Plain(Box::new(diag)));
                    return;
                }
            }
        }
        // An orphan-rule namespace registration (E0210/E0117): a crate registering wiring into a
        // foreign namespace keyed on a foreign key. Reword the raw coherence error — which names
        // the machinery parameter and frames a CGP wiring decision as a bare coherence rule — into
        // a `[CGP-E022]` header naming the namespace and key, re-aiming the caret at the offending
        // macro alone (its "uncovered type parameter" label no longer applies), and carrying the
        // ownership-based fix in a `help`.
        if let Some(conflict) = self.orphan_conflict(&diag)
            && let Some(primary_span) = diag.span.primary_span()
        {
            replace_header(&mut diag, plan_orphan_conflict(&conflict));
            diag.span = MultiSpan::from_span(primary_span);
            diag.children
                .push(subdiag(Sublevel::Help, orphan_conflict_help(&conflict)));
            // A rewritten diagnostic: bare `@…` paths (a path key renders without the `Path!`
            // wrapper).
            self.postprocess(&mut diag, Origin::Rewritten);
            self.record_cgp_spans(&diag);
            self.push(BufEntry::Plain(Box::new(diag)));
            return;
        }
        // A structural mistake in a wiring or its checks, such as a check table listing one entry
        // twice: the compiler names only the generated items, so reword the header to name the
        // mistake and carry the fix in a `help`, keeping rustc's carets on the two sites.
        if let Some(mistake) = self.structural_mistake(&diag) {
            // One mistake can produce several errors (a loop's key variable and its value variable
            // are each unconstrained, in an entry and in its `IsProviderFor` twin); report it once.
            if self.structural_reported.contains(&mistake) {
                return;
            }
            self.structural_reported.push(mistake.clone());
            replace_header(&mut diag, plan_structural_mistake(&mistake));
            if matches!(
                mistake,
                StructuralMistake::LoopKeyWithoutVariable { .. }
                    | StructuralMistake::GenericDefaultImpl { .. }
            ) && let Some(primary) = diag.span.primary_span()
            {
                // rustc's suggestions to remove the parameter or add it to the context are not the
                // fix; keep the caret on the parameter alone.
                diag.span = MultiSpan::from_span(primary);
                diag.children.clear();
                diag.suggestions = Suggestions::Enabled(Vec::new());
            }
            if matches!(mistake, StructuralMistake::DuplicateCheckTrait { .. }) {
                // rustc's labels and note name the generated check trait; label the two tables
                // instead, the reported one as the second.
                diag.span = check_table_span(&diag.span);
                diag.children.clear();
            }
            diag.children
                .push(subdiag(Sublevel::Help, structural_mistake_help(&mistake)));
            self.postprocess(&mut diag, Origin::Rewritten);
            self.record_cgp_spans(&diag);
            self.push(BufEntry::Plain(Box::new(diag)));
            return;
        }
        // A trait called in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared via `#[uses(…)]`:
        // its method cannot resolve on the generated `__Context__` generic, and rustc reports a
        // vague `E0599` pointing at a transitive `HasField` bound. Reword it to name the trait
        // and carry the `#[uses(…)]` fix in a `help`, keeping the caret on the failing call. Recording
        // the span drops the unsized-`[u8]` cascade the failed method resolution trails on the same
        // expression.
        if let Some(undeclared) = self.undeclared_trait(&diag)
            && let Some(primary_span) = diag.span.primary_span()
        {
            replace_header(&mut diag, plan_undeclared_trait(&undeclared));
            diag.span = MultiSpan::from_span(primary_span);
            diag.children
                .push(subdiag(Sublevel::Help, undeclared_trait_help(&undeclared)));
            self.postprocess(&mut diag, Origin::Rewritten);
            self.record_cgp_spans(&diag);
            self.push(BufEntry::Plain(Box::new(diag)));
            return;
        }
        // A higher-order provider calling an inner provider it never imported with `#[use_provider]`:
        // the inner parameter is unbounded, so rustc reports a vague `E0599` that leaks `__Context__`
        // and suggests the wrong (consumer-trait) bound. Reword it to name the inner provider and
        // carry the `#[use_provider(…)]` fix in a `help`, keeping the caret on the failing call.
        if let Some(missing) = self.missing_use_provider(&diag)
            && let Some(primary_span) = diag.span.primary_span()
        {
            replace_header(&mut diag, plan_missing_use_provider(&missing));
            diag.span = MultiSpan::from_span(primary_span);
            diag.children
                .push(subdiag(Sublevel::Help, missing_use_provider_help(&missing)));
            self.postprocess(&mut diag, Origin::Rewritten);
            self.record_cgp_spans(&diag);
            self.push(BufEntry::Plain(Box::new(diag)));
            return;
        }
        if is_question_mark_cascade(&diag) && self.overlaps_cgp_failure(&diag) {
            // A downstream `?`-operator cascade of a CGP wiring failure already reported at this
            // expression: it restates the failure in `Try` terms and dumps the unresolved projected
            // type, adding nothing. Drop it (cargo re-counts emitted diagnostics, so the "N errors"
            // summary stays honest). A `?` error with no CGP failure on its expression is untouched.
            // The check runs before resolution, since the call-site anchor would otherwise
            // re-anchor the cascade onto the failing call it overlaps.
            //
            // Only the `?` cascade is dropped, never a `[T]: Sized` one: an unsized error the failed
            // method/trait resolution trails can land off the failing expression (on the binding
            // pattern, or a later statement the unresolved type flows into), where a span-overlap
            // check misses it and a broader check would risk suppressing an unrelated error — so
            // those are left in place rather than dropped unreliably.
            return;
        }
        // A resolvable wiring failure is transformed around its dependency tree(s); when the
        // resolver declines, the wiring-message rename runs as the first fallback pass. A resolved
        // failure also yields its span-independent cause signature, for the de-duplication below.
        let mut class_headers: Vec<String> = Vec::new();
        let (rewritten, cause_sig, resolution) =
            if let Some((resolved, span, at_call)) = self.try_resolve(&diag) {
                // The downstream check re-report of a `#[cgp_impl]` consumer-trait mistake (its
                // provider struct failing `NotAProvider` because its impl targets the wrong trait)
                // is a pure consequence of the `[CGP-E032]` already shown, so drop it. A failing
                // requirement of that impl resolves to a cause of its own and is kept.
                if self.is_cgp_impl_misuse_check_report(&resolved) {
                    return;
                }
                let sig = cause_signature(&resolved);
                let transformed = self.transform_resolved(&mut diag, &resolved, span, at_call);
                // A failure whose headline gives way to its cause coalesces at flush with every
                // failure it shares a root cause with; a mismatch or a kept rustc headline emits on
                // its own. Every resolution defers its note to the flush, so all of them elide
                // against one another.
                class_headers = transformed.headers;
                let coalescing = transformed.coalesces;
                (
                    true,
                    Some(sig),
                    Some((resolved, transformed.note, coalescing)),
                )
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
                // A wiring overflow is worded by why its lookup never resolves; one the walk does
                // not explain (it follows from an unresolved type, say) keeps rustc's wording.
                let overflow = self.wiring_overflow(&diag);
                let mut changed =
                    (diag.code != Some(E0275) || overflow.is_some()) && self.rewrite(&mut diag);
                if let Some(overflow) = &overflow
                    && let Some(header) = main_message_text(&diag)
                        .and_then(|text| rewrite_wiring_overflow(text, &self.names, overflow))
                {
                    diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
                    changed = true;
                    // The note pointing at the generated `__Check…` trait names something the user
                    // never wrote, whose location the kept caret already covers, and rustc's advice
                    // to raise the recursion limit cannot fix a lookup that never ends.
                    diag.children.retain(|child| {
                        !child.messages.iter().any(|(message, _)| {
                            matches!(
                                message,
                                DiagMessage::Str(text)
                                    if text.contains("__Check")
                                        || text.contains("__CanUse")
                                        || is_recursion_limit_help_text(text)
                            )
                        })
                    });
                    diag.children
                        .push(subdiag(Sublevel::Help, wiring_overflow_help(overflow)));
                }
                (changed, None, None)
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
        // own transformed diagnostics are de-duplicated; an untouched `rustc` error always passes
        // through. cargo re-counts the diagnostics the emitter produces, so a suppressed re-report
        // drops out of its "N errors" summary as well, keeping the count consistent. A resolved
        // failure records its class headlines, not the cause it displays, since a declined re-report
        // of the same failure is worded with a class sentence.
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
        let original = diag.clone();
        let armed = self.test_panic;
        let transformed = contain(|| {
            if original.is_error() {
                inject(armed, TestPanic::Transform);
            }
            self.transform(diag, &original);
        });
        if let Err(detail) = transformed {
            let mut diag = original;
            diag.children.push(subdiag(
                Sublevel::Note,
                internal_error_note(&detail, BUG_REPORT_URL),
            ));
            self.push(BufEntry::Plain(Box::new(diag)));
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
        let armed = self.test_panic;
        let to_emit = match contain(|| {
            inject(armed, TestPanic::Flush);
            self.render(&buffer)
        }) {
            Ok(rendered) => rendered,
            Err(detail) => {
                let note = internal_error_note(&detail, BUG_REPORT_URL);
                buffer
                    .into_iter()
                    .map(|entry| entry.untransformed(Some(&note)))
                    .collect()
            }
        };
        let mut output = self.output();
        for diag in to_emit {
            output.inner.emit_diagnostic(diag);
        }
    }
}
