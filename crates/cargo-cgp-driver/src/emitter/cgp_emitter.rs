//! The wrapping [`Emitter`] that transforms CGP diagnostics before delegating.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, rewrite_message, rewrite_required_for, wiring_overflow_help,
};
use cargo_cgp_error_processing::{
    Cause, CgpImplMisuse, ChainNode, DedupLedger, DiagKind, Leaf, MissingUseProvider,
    OrphanConflict, PendingNote, Resolved, UndeclaredCapability, cause_only_signature,
    cause_signature, cgp_impl_misuse_help, coalesce_underived_fields, consumer_header,
    fix_help_messages, is_method_bounds_text, is_unbounded_type_param_item_text,
    mentions_orphan_param_text, missing_use_provider_help, orphan_conflict_help,
    plan_cgp_impl_misuse, plan_missing_use_provider, plan_orphan_conflict, plan_resolved,
    plan_undeclared_capability, plan_wiring_conflict, postprocess_message,
    undeclared_capability_help, wiring_conflict_help,
};
use rustc_errors::codes::{
    E0107, E0117, E0119, E0186, E0207, E0210, E0271, E0275, E0277, E0308, E0425, E0599,
};
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
use crate::resolve::{self, ConflictAction, ConflictTrait, DetectedCgpImplMisuse, ResolveCache};

/// The wrapping [`Emitter`] that transforms CGP diagnostics before delegating to the real
/// inner emitter. Generic over the inner emitter `E` so the driver can wrap whichever the
/// compiler's default would build for the active error format — a `JsonEmitter` or an
/// `AnnotateSnippetEmitter` — and render like vanilla `rustc` in either.
pub struct CgpEmitter<E: Emitter> {
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
    /// Every diagnostic this emitter keeps, in arrival order, held until [`Drop`] so that separate
    /// failures sharing one root cause can be coalesced into a single block. The compiler streams
    /// diagnostics one at a time with no "end of compilation" hook, so listing every consumer a
    /// mistake breaks in one headline is only possible once they have all arrived — which is why
    /// the buffer is flushed from `Drop` (the inner emitter is still alive then), not eagerly. A
    /// [`BufEntry::Coalescible`] failure joins the group of its cause-only signature; everything
    /// else — an untouched `rustc` error, a conflict, a declined fallback — is a
    /// [`BufEntry::Plain`] emitted verbatim at its original position, so ordering is preserved.
    buffer: Vec<BufEntry>,
    /// The `#[cgp_impl]` header-trait mistakes in the crate — a consumer trait or a non-CGP trait
    /// named where the provider trait belongs — detected once from the compiler and reused. `None`
    /// until first computed under an available `TyCtxt` (the macro-lowering errors this recognizes
    /// arrive before trait solving, so the first candidate may precede one); an empty `Vec` means
    /// "computed, none found". Owned data (names plus `Copy` spans), so it outlives the `TyCtxt`.
    cgp_impl_misuses: Option<Vec<DetectedCgpImplMisuse>>,
}

/// One buffered diagnostic awaiting the [`Drop`]-time flush.
enum BufEntry {
    /// Emitted verbatim at its arrival position. Boxed, like the variant below, so the buffer's
    /// element stays a pointer rather than a whole `DiagInner`.
    Plain(Box<DiagInner>),
    /// A typed-resolution failure, whose `root cause:` note is rendered at *flush* rather than on
    /// arrival: only there is the emission order known, so a note can `(*)`-elide the subtrees an
    /// earlier block already drew. `diag` therefore carries the header and `help`s but no note yet.
    ///
    /// `sig` is `Some` for a consumer-trait failure, which coalesces with others of the same
    /// [`cause_only_signature`]: a group of one emits `diag` alone, a group of several emits a single
    /// merged block naming every affected consumer at the position of the first. It is `None` for a
    /// resolution that never coalesces — a wrapper trait, a mismatch, a provider-side check.
    Resolved(Box<ResolvedEntry>),
}

/// The payload of a [`BufEntry::Resolved`].
struct ResolvedEntry {
    sig: Option<String>,
    resolved: Resolved,
    note: PendingNote,
    diag: DiagInner,
}

/// The subtrees a flush has already drawn, shared across its blocks so a later one `(*)`-elides
/// what an earlier one showed rather than repeating it.
type ChainNodeSet = HashSet<ChainNode>;

/// Whether a resolution is a *consumer-trait* failure on the checked context itself — the only shape
/// that coalesces. A field-type mismatch, a provider-side check, or a foreign-wrapper failure is
/// left to emit on its own.
fn is_consumer_shaped(resolved: &Resolved) -> bool {
    resolved.consumers_are_cgp && resolved.subject_is_context && !resolved.consumers.is_empty()
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
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            names: ComponentNameMap::new(build_name_map_from_tls),
            dedup: DedupLedger::new(),
            resolve_cache: ResolveCache::new(),
            cgp_spans: Vec::new(),
            buffer: Vec::new(),
            cgp_impl_misuses: None,
        }
    }

    /// Build the one merged block for a coalesced group of several consumer failures sharing a root
    /// cause: a `[CGP-E001]` header listing every affected consumer trait, a caret at each failing
    /// entry, and one root-cause note built by folding *every* member's causes into a single
    /// dependency graph — so a consumer whose chain runs through another collapses into it, while
    /// independent chains to the shared cause render side by side, and no member's chain is dropped.
    /// The single-consumer header rustc's per-entry rendering produced — even a provider-side
    /// `[CGP-E002]` one — is dropped in favour of the consumer form, since a `check_components!`
    /// entry failing *is* the consumer trait failing.
    fn merged_diag(
        &self,
        resolveds: &[&Resolved],
        diags: &[&DiagInner],
        seen: &mut ChainNodeSet,
    ) -> DiagInner {
        let first = resolveds[0];
        let mut consumers: Vec<String> = Vec::new();
        for resolved in resolveds {
            for consumer in &resolved.consumers {
                if !consumers.contains(consumer) {
                    consumers.push(consumer.clone());
                }
            }
        }
        let merged = Resolved {
            context: first.context.clone(),
            consumers,
            consumers_are_cgp: true,
            subject_is_context: true,
            // Only the consumers and context matter for the header; the note is built from the
            // causes below.
            causes: Vec::new(),
        };

        // Every cause across the coalesced consumers. The dependency graph then merges what they
        // share — a consumer whose chain runs through another collapses into it, while independent
        // chains to one cause render side by side — so no chain is dropped and every consumer appears.
        let causes: Vec<Cause> = resolveds
            .iter()
            .flat_map(|resolved| resolved.causes.iter().cloned())
            .collect();
        let causes = coalesce_underived_fields(&causes);
        let mut children: Vec<_> = fix_help_messages(&causes)
            .into_iter()
            .map(|help| subdiag(Level::Help, help))
            .collect();
        children.extend(
            PendingNote {
                causes,
                header_leaf: None,
            }
            .render(seen)
            .into_iter()
            .map(|note| subdiag(Level::Note, note)),
        );

        let mut diag = diags[0].clone();
        diag.messages = vec![(
            DiagMessage::Str(consumer_header(&merged).into()),
            Style::NoStyle,
        )];
        // One caret per failing entry, in arrival (check) order.
        let mut span = MultiSpan::new();
        for member in diags {
            if let Some(primary) = member.span.primary_span() {
                span.push_primary_span(primary);
            }
        }
        diag.span = span;
        diag.children = children;
        diag.suggestions = Suggestions::Enabled(Vec::new());
        // The header, notes, and helps are freshly built, so post-process them (resugar `Symbol!`,
        // strip CGP prefixes) as the streaming path does for a rewritten diagnostic.
        self.postprocess(&mut diag, true);
        diag
    }

    /// Flush the buffer to the inner emitter, coalescing each group of [`BufEntry::Coalescible`]
    /// failures that share a cause-only signature into one block at the position of its first
    /// member. Called only from [`Drop`], where the inner emitter is still alive.
    fn flush(&mut self) {
        let buffer = std::mem::take(&mut self.buffer);

        // Group the coalescible members by signature, keeping their arrival order.
        let mut groups: HashMap<&str, Vec<usize>> = HashMap::new();
        for (index, entry) in buffer.iter().enumerate() {
            if let BufEntry::Resolved(entry) = entry
                && let Some(sig) = &entry.sig
            {
                groups.entry(sig.as_str()).or_default().push(index);
            }
        }

        // The subtrees drawn so far, threaded through every block in emission order so a later one
        // truncates at what an earlier one already showed. This is why notes are rendered here
        // rather than on arrival: only now is that order known.
        let mut seen = ChainNodeSet::new();
        let mut emitted: HashSet<&str> = HashSet::new();
        let mut to_emit: Vec<DiagInner> = Vec::new();
        for entry in &buffer {
            match entry {
                BufEntry::Plain(diag) => to_emit.push((**diag).clone()),
                BufEntry::Resolved(entry) => {
                    let Some(sig) = &entry.sig else {
                        // A resolution that never coalesces: render its own note and emit it here.
                        let mut diag = entry.diag.clone();
                        self.append_note(&mut diag, &entry.note, &mut seen);
                        to_emit.push(diag);
                        continue;
                    };
                    if !emitted.insert(sig.as_str()) {
                        continue;
                    }
                    let members = &groups[sig.as_str()];
                    if members.len() == 1 {
                        let BufEntry::Resolved(only) = &buffer[members[0]] else {
                            unreachable!("indices came from the coalescible entries");
                        };
                        let mut diag = only.diag.clone();
                        self.append_note(&mut diag, &only.note, &mut seen);
                        to_emit.push(diag);
                    } else {
                        let (resolveds, diags): (Vec<&Resolved>, Vec<&DiagInner>) = members
                            .iter()
                            .map(|&index| match &buffer[index] {
                                BufEntry::Resolved(member) => (&member.resolved, &member.diag),
                                _ => unreachable!("indices came from the coalescible entries"),
                            })
                            .unzip();
                        to_emit.push(self.merged_diag(&resolveds, &diags, &mut seen));
                    }
                }
            }
        }

        for diag in to_emit {
            self.inner.emit_diagnostic(diag);
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

    /// Recognize `diag` as an undeclared-capability failure — a CGP capability called in a
    /// `#[cgp_fn]`/`#[cgp_impl]` body without being declared via `#[uses(…)]`, so its method cannot
    /// resolve on the generated `__Context__` generic. Returns the capability to declare, or `None`
    /// when it is not one. Gated to the method-bounds `E0599` shape, then confirmed structurally by
    /// [`resolve::detect_undeclared_capability`] (a generated blanket impl with a bare-parameter
    /// `Self`, a capability-trait method call, and no matching `where` bound).
    fn undeclared_capability(&self, diag: &DiagInner) -> Option<UndeclaredCapability> {
        if diag.code != Some(E0599) || !main_message_text(diag).is_some_and(is_method_bounds_text) {
            return None;
        }
        let primary_span = diag.span.primary_span()?;
        let spans = diagnostic_spans(diag);
        rustc_middle::ty::tls::with_opt(|tcx| {
            resolve::detect_undeclared_capability(tcx?, primary_span, &spans)
        })
    }

    /// Recognize a missing-`#[use_provider]` failure — a higher-order provider calling an inner
    /// provider (`Inner::method(self)`) it never imported, so the inner parameter is unbounded and
    /// the associated-function call cannot resolve. Returns the inner provider and the provider trait
    /// to import it as, or `None`. Gated to the "no associated item for a type parameter" `E0599`
    /// shape — reported during typeck of the calling body, so the detector's queries are cached and
    /// safe (unlike the resolution-class `E0599` emitted mid-`predicates_of`) — then confirmed
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
    /// consequence of the mistake already reported as `[CGP-E013]` (the provider struct cannot be a
    /// provider because its impl targets the wrong trait), so it is suppressed rather than shown as
    /// its own `[CGP-E111]` block.
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

    /// Rebuild the trailing "detailed explanations" footer when this crate had a `#[cgp_impl]` header
    /// mistake. rustc builds that footer from every error code it *registered* as diagnostics
    /// arrived — which includes the cascade siblings this emitter then suppressed
    /// (`E0425`/`E0186`/`E0207`/`E0277`) — so left alone it would list `rustc --explain` codes for
    /// errors no longer shown, contradicting the single error that survives. The footer arrives last
    /// (from `print_error_count`, after every error is buffered), so it is rebuilt from the codes
    /// still in the buffer. Returns the replacement diagnostics (empty to drop the footer), or `None`
    /// when `diag` is not a footer or the feature is inactive — so no output without this mistake is
    /// touched.
    fn rebuilt_explain_footer(&self, diag: &DiagInner) -> Option<Vec<DiagInner>> {
        match self.cgp_impl_misuses.as_deref() {
            Some(misuses) if !misuses.is_empty() => {}
            _ => return None,
        }
        if diag.level() != Level::FailureNote {
            return None;
        }
        let text = main_message_text(diag)?;
        let is_list = text.starts_with("Some errors have detailed explanations:");
        let is_pointer = text.starts_with("For more information about");
        if !is_list && !is_pointer {
            return None;
        }
        // The rust codes of the error diagnostics still in the buffer — what is actually shown.
        let mut codes: Vec<String> = self
            .buffer
            .iter()
            .filter_map(|entry| match entry {
                BufEntry::Plain(diag) => diag.code,
                BufEntry::Resolved(entry) => entry.diag.code,
            })
            .map(|code| code.to_string())
            .collect();
        codes.sort();
        codes.dedup();

        // The "Some errors …" list line is only right for two or more surviving codes; otherwise the
        // singular pointer line below carries the sole code, so drop the list line.
        if is_list {
            if codes.len() < 2 {
                return Some(Vec::new());
            }
            return Some(vec![footer_note(
                diag,
                &format!(
                    "Some errors have detailed explanations: {}.",
                    codes.join(", ")
                ),
            )]);
        }
        // The pointer line: singular when one code survives, plural when several, dropped when none.
        let Some(first) = codes.first() else {
            return Some(Vec::new());
        };
        let message = if codes.len() == 1 {
            format!("For more information about this error, try `rustc --explain {first}`.")
        } else {
            format!("For more information about an error, try `rustc --explain {first}`.")
        };
        Some(vec![footer_note(diag, &message)])
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
            // The next resort re-reads the failing *call expression* itself — the anchor for a
            // consumer-method `E0277` whose spans never touch the context's definition (a
            // `Code`-dispatched handler pipeline that matches unconditionally), and for a direct
            // call to a `#[cgp_fn]` capability method. A resolution from here is flagged, so the
            // header is worded from the trait the call needs rather than from whichever provider
            // bound rustc's headline stopped on.
            if let Some(resolved) = resolve::resolve_call_site(tcx, cache, &spans) {
                return Some((resolved, true));
            }
            // Last: a `#[cgp_fn]` / `#[blanket_trait]` capability trait the diagnostic names in its
            // spans, required through a `where` bound or supertrait rather than a direct call. This
            // is gated to `E0277` — a capability *used as a bound* — deliberately: an `E0599` method
            // call is the call-site anchor's domain, and a *generic consumer* method call whose deep
            // capability bound is a note (not the failure the diagnostic is about) must stay declined
            // when its dispatch parameter is unrecoverable, rather than latch onto that transitive
            // capability (see `generic_consumer_unwritten_arg`).
            if diag.code == Some(E0277)
                && let Some(resolved) = resolve::resolve_use_site_capability(tcx, cache, &spans)
            {
                return Some((resolved, false));
            }
            None
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
    ) -> PendingNote {
        let kind = match diag_kind(diag) {
            kind if at_call && kind != DiagKind::TypeMismatch => DiagKind::MethodNotFound,
            kind => kind,
        };
        let plan = plan_resolved(kind, main_message_text(diag), resolved, &self.names);

        if let Some(header) = plan.header {
            diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
            // Re-aim the caret at the failing entry alone: the original span labels restate the
            // replaced message, so they no longer apply.
            diag.span = MultiSpan::from_span(span);
        }

        // The `help`s are applied now; the `root cause:` note is not, because rendering it needs the
        // emission order the flush establishes (see [`PendingNote`]). It is appended there, after
        // these children, which preserves the help-then-note order this built directly before.
        diag.children = plan
            .helps
            .into_iter()
            .map(|help| subdiag(Level::Help, help))
            .collect();
        // Drop rustc's structured suggestions along with its notes — for a use-site failure
        // that includes the misleading "use associated function syntax instead".
        diag.suggestions = Suggestions::Enabled(vec![]);

        plan.note
    }

    /// Render a deferred [`PendingNote`] against the `seen` set shared by this flush and append it to
    /// `diag`. The note is post-processed on its own, since the rest of the diagnostic was already
    /// processed when it arrived and re-running the chain over it could compound.
    fn append_note(&self, diag: &mut DiagInner, note: &PendingNote, seen: &mut ChainNodeSet) {
        // A resolver-built note never carries the `` `HasField<…>` is not implemented `` clause the
        // missing-field reword keys on (see docs/implementation/error-processing.md), so that branch
        // has nothing to match and the flag it needs is moot here.
        const HAS_FIELD_IMPLS: bool = false;
        for text in note.render(seen) {
            let text = postprocess_message(&text, HAS_FIELD_IMPLS, true).unwrap_or(text);
            diag.children.push(subdiag(Level::Note, text));
        }
    }
}

impl<E: Emitter> Emitter for CgpEmitter<E> {
    fn emit_diagnostic(&mut self, mut diag: DiagInner) {
        // The trailing "detailed explanations" footer, rebuilt when a `#[cgp_impl]` header mistake
        // had its cascade suppressed, so it lists `rustc --explain` codes only for errors still
        // shown. Confined to crates with the mistake, so no other output is affected.
        if let Some(replacement) = self.rebuilt_explain_footer(&diag) {
            self.buffer.extend(
                replacement
                    .into_iter()
                    .map(|diag| BufEntry::Plain(Box::new(diag))),
            );
            return;
        }
        // A `#[cgp_impl]` header naming the wrong trait — the component's consumer trait, or a
        // non-CGP trait — makes the macro generate an inside-out impl of the wrong trait, producing
        // a burst of cryptic macro-lowering errors (E0425/E0107/E0186/E0207) plus a downstream check
        // failure, none naming the mistake. Detect it structurally (once) and reshape the E0107
        // whose caret is on the misused trait name into a `[CGP-E013]`/`[CGP-E014]` header with the
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
                    diag.children = vec![subdiag(Level::Help, cgp_impl_misuse_help(&misuse))];
                    diag.suggestions = Suggestions::Enabled(Vec::new());
                    self.postprocess(&mut diag, true);
                    // Drop any sibling of this impl already buffered: E0425 (name resolution)
                    // arrives before this E0107 (type lowering), possibly before a `TyCtxt` was in
                    // scope to detect the mistake, so it was buffered verbatim.
                    self.buffer
                        .retain(|entry| !is_cgp_impl_sibling_entry(entry, impl_span, macro_span));
                    self.record_cgp_spans(&diag);
                    self.buffer.push(BufEntry::Plain(Box::new(diag)));
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
                        diag.children.push(subdiag(Level::Help, help));
                    }
                    // A rewritten diagnostic: bare `@…` paths.
                    self.postprocess(&mut diag, true);
                    self.record_cgp_spans(&diag);
                    self.buffer.push(BufEntry::Plain(Box::new(diag)));
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
            self.buffer.push(BufEntry::Plain(Box::new(diag)));
            return;
        }
        // A capability called in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared via `#[uses(…)]`:
        // its method cannot resolve on the generated `__Context__` generic, and rustc reports a
        // vague `E0599` pointing at a transitive `HasField` bound. Reword it to name the capability
        // and carry the `#[uses(…)]` fix in a `help`, keeping the caret on the failing call. Recording
        // the span drops the unsized-`[u8]` cascade the failed method resolution trails on the same
        // expression.
        if let Some(undeclared) = self.undeclared_capability(&diag)
            && let Some(primary_span) = diag.span.primary_span()
        {
            replace_header(&mut diag, plan_undeclared_capability(&undeclared));
            diag.span = MultiSpan::from_span(primary_span);
            diag.children.push(subdiag(
                Level::Help,
                undeclared_capability_help(&undeclared),
            ));
            self.postprocess(&mut diag, true);
            self.record_cgp_spans(&diag);
            self.buffer.push(BufEntry::Plain(Box::new(diag)));
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
                .push(subdiag(Level::Help, missing_use_provider_help(&missing)));
            self.postprocess(&mut diag, true);
            self.record_cgp_spans(&diag);
            self.buffer.push(BufEntry::Plain(Box::new(diag)));
            return;
        }
        // A resolvable wiring failure is transformed around its dependency tree(s); when the
        // resolver declines, the wiring-message rename runs as the first fallback pass. A resolved
        // failure also yields its span-independent cause signature, for the de-duplication below.
        let (rewritten, cause_sig, resolution) =
            if let Some((resolved, span, at_call)) = self.try_resolve(&diag) {
                // The downstream check re-report of a `#[cgp_impl]` consumer-trait mistake (its
                // provider struct failing `NotAProvider` because its impl targets the wrong trait)
                // is a pure consequence of the `[CGP-E013]` already shown, so drop it.
                if self.is_cgp_impl_misuse_check_report(&resolved) {
                    return;
                }
                let sig = cause_signature(&resolved);
                let note = self.transform_resolved(&mut diag, &resolved, span, at_call);
                // A consumer-trait failure joins its cause-only group for coalescing at flush; any
                // other shape (a field mismatch, a provider check) emits on its own — but every
                // resolution defers its note to the flush, so all of them elide against one another.
                let coalescing = is_consumer_shaped(&resolved);
                (true, Some(sig), Some((resolved, note, coalescing)))
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
                (changed, None, None)
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
            //
            // Only the `?` cascade is dropped, never a `[T]: Sized` one: an unsized error the failed
            // method/trait resolution trails can land off the failing expression (on the binding
            // pattern, or a later statement the unresolved type flows into), where a span-overlap
            // check misses it and a broader check would risk suppressing an unrelated error — so
            // those are left in place rather than dropped unreliably.
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
        // Buffer rather than emit now: a resolution has its `root cause:` note rendered at flush,
        // where the emission order lets it elide what an earlier block drew, and a consumer failure
        // additionally joins its cause-only group for coalescing there. Everything else is emitted
        // verbatim in place. The flush happens in `Drop`, the only point after every diagnostic has
        // arrived.
        match resolution {
            Some((resolved, note, coalescing)) => {
                let sig = coalescing.then(|| cause_only_signature(&resolved));
                self.buffer.push(BufEntry::Resolved(Box::new(ResolvedEntry {
                    sig,
                    resolved,
                    note,
                    diag,
                })));
            }
            None => self.buffer.push(BufEntry::Plain(Box::new(diag))),
        }
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

impl<E: Emitter> Drop for CgpEmitter<E> {
    /// Flush the buffered diagnostics when the compiler drops the emitter. This runs during the
    /// `DiagCtxt`'s own teardown, *after* every diagnostic has been handed to `emit_diagnostic` but
    /// while the inner emitter (a field of `self`, dropped only after this returns) is still alive —
    /// the one place a "list every affected consumer" block can be built, since the `Emitter` trait
    /// offers no end-of-compilation hook. Diagnostics were counted by the `DiagCtxt` as they
    /// arrived, so deferring their *rendering* to here does not change the error count.
    fn drop(&mut self) {
        self.flush();
    }
}
