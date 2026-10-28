//! Stage 17: the typed root-cause resolver, and the text rewrite when it declines.
//!
//! A diagnostic no earlier stage took is offered to the typed resolver. A resolution replaces the
//! diagnostic's header and helps and leaves its `root cause:` note for the flush; when the resolver
//! declines, the wiring-message rename and the overflow rewrite run as the fallback. Either way the
//! emitter's final tail post-processes, de-duplicates, and buffers the result.

use cargo_cgp_error_processing::rewrite::{
    ComponentNameMap, rewrite_message, rewrite_required_for, rewrite_wiring_overflow,
};
use cargo_cgp_error_processing::{
    DiagKind, PendingNote, Resolved, WiringOverflow, cause_signature, is_method_bounds_text,
    is_recursion_limit_help_text, plan_resolved, wiring_overflow_help,
};
use rustc_errors::codes::{E0271, E0275, E0277, E0308, E0599};
use rustc_errors::{DiagInner, DiagMessage, MultiSpan, Style, Sublevel, Suggestions};
use rustc_span::Span;

use crate::emitter::edit::{
    diag_kind, diagnostic_spans, is_no_matching_item, is_question_mark_cascade, main_message_text,
    mentions_wiring, rewrite_messages, strip_method_probe_advice, subdiag,
};
use crate::emitter::stage::CgpImplMisuseStage;
use crate::resolve::{self, ResolveCtx};

/// What stage 17 made of a diagnostic no earlier stage took.
pub enum Resolution {
    /// The downstream check re-report of a `#[cgp_impl]` consumer-trait mistake, a pure consequence
    /// of the `[CGP-E032]` already shown.
    Drop,
    /// A resolved failure: the diagnostic now carries the plan's header and helps.
    Resolved(Box<ResolvedFailure>),
    /// The resolver declined; `rewritten` says whether the text fallback changed the diagnostic.
    Declined { rewritten: bool },
}

/// A failure the resolver traced, with what the emitter's final tail needs from it.
pub struct ResolvedFailure {
    pub resolved: Resolved,
    /// The span-independent signature of the cause, for the de-duplication ledger.
    pub cause_signature: String,
    pub transformed: TransformedResolution,
}

/// What [`transform_resolved`] leaves for the rest of the transform: the note the flush renders,
/// the class headline the de-duplication ledger records, and whether the block may coalesce with
/// others sharing its cause.
pub struct TransformedResolution {
    pub note: PendingNote,
    /// The class headline, and the provider form a declined re-report may carry instead.
    pub headers: Vec<String>,
    pub coalesces: bool,
}

/// Stage 17: transform a resolvable wiring failure around its dependency tree, or, when the
/// resolver declines, run the wiring-message rename and the overflow rewrite as the fallback.
/// `cx` is `None` when no compiler handle is in scope, which the text fallback does not need.
/// `cgp_impl_misuse` is stage 2, whose detected mistakes make a failing check's re-report a drop.
pub fn resolve_or_rewrite(
    cx: Option<ResolveCtx<'_, '_>>,
    diag: &mut DiagInner,
    names: &ComponentNameMap,
    cgp_impl_misuse: &CgpImplMisuseStage,
) -> Resolution {
    if let Some((resolved, span, at_call)) = try_resolve(cx, diag) {
        // The downstream check re-report of a `#[cgp_impl]` consumer-trait mistake (its provider
        // struct failing `NotAProvider` because its impl targets the wrong trait) is dropped. A
        // failing requirement of that impl resolves to a cause of its own and is kept.
        if cgp_impl_misuse.is_check_report(&resolved) {
            return Resolution::Drop;
        }
        let cause_signature = cause_signature(&resolved);
        let transformed = transform_resolved(diag, &resolved, span, at_call, names);
        return Resolution::Resolved(Box::new(ResolvedFailure {
            resolved,
            cause_signature,
            transformed,
        }));
    }
    // A CGP consumer-method `E0599` the resolver declined still carries rustc's method-probe
    // advice: the associated-function framing and the "use associated function syntax instead"
    // suggestion, both artifacts of the provider's `self`-less methods, the second actively wrong.
    // Strip that noise so the unmet wiring bound the diagnostic also names is not outranked by it.
    if diag.code == Some(E0599)
        && main_message_text(diag).is_some_and(is_method_bounds_text)
        && mentions_wiring(diag)
    {
        strip_method_probe_advice(diag);
    }
    // A wiring overflow is worded by why its lookup never resolves; one the walk does not explain
    // (it follows from an unresolved type, say) keeps rustc's wording.
    let overflow = wiring_overflow(cx, diag);
    let mut rewritten = (diag.code != Some(E0275) || overflow.is_some()) && rewrite(diag, names);
    if let Some(overflow) = &overflow
        && let Some(header) =
            main_message_text(diag).and_then(|text| rewrite_wiring_overflow(text, names, overflow))
    {
        diag.messages = vec![(DiagMessage::Str(header.into()), Style::NoStyle)];
        rewritten = true;
        // The note pointing at the generated `__Check…` trait names something the user never
        // wrote, whose location the kept caret already covers, and rustc's advice to raise the
        // recursion limit cannot fix a lookup that never ends.
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
    Resolution::Declined { rewritten }
}

/// Resolve `diag`'s failure to its root-cause dependency tree(s), or `None` when the resolver
/// cannot trace it to a CGP component failure (so the caller falls back to the in-place text
/// rewrite). A candidate is every diagnostic that reports an unmet obligation (`E0271`, `E0277`,
/// and a *method-bounds* `E0599`), whether or not it names a CGP trait — because a failure *not*
/// worded in CGP terms can still be a consequence of a CGP component failing (a manual `impl`
/// that forwards to a wired method, a downstream trait bound that needs it), and [`resolve`]
/// traces the dependency chain to find out. It yields `None` for everything whose chain does not
/// reach a CGP cause. Returns the primary span alongside the resolution so the caret can be
/// re-aimed at the entry.
///
/// An `E0275` overflow is no candidate: the walk explains it only by a cut, a cycle or a lookup
/// that grows, which [`wiring_overflow`] words, and a leaf the walk stops
/// at instead is a bound whose own evaluation overflowed, which no root cause states truly.
///
/// Two more shapes reach the data-call anchor alone: the method probe's "no method named `…`"
/// `E0599`, and an `E0308` on a call's argument, each of which a call to one of CGP's
/// extensible-data operations can fail with before its bounds are checked; an `E0308` is taken
/// only when the walk finds the field or variant state the operation rejects.
///
/// The `E0599` arm is otherwise narrowed to the "the method `…` exists … but its trait bounds
/// were not satisfied" shape, the consumer-method call the use-site anchor handles. A
/// *resolution*-class `E0599` (`no variant named …`, `no associated item …`) is not a wiring
/// failure and, worse, is emitted *during* type lowering / `clauses_of`, while that query is
/// mid-flight: running the resolver's trait solver on it re-forces an emitting query and
/// re-enters the already-held `DiagCtxt` lock, aborting the compiler (`lock was already held`).
/// Declining such an `E0599` before any solving both keeps the tool from crashing and is
/// correct, since the resolver has nothing to say about a name-resolution error.
/// (`E0271`/`E0277` are trait-solving failures reported after collection, where the queries the
/// solver forces are already cached.)
pub fn try_resolve(
    cx: Option<ResolveCtx<'_, '_>>,
    diag: &DiagInner,
) -> Option<(Resolved, Span, bool)> {
    // A method-bounds `E0599` (not a resolution-class one) is the `E0599` the anchors handle,
    // beside the data-call shape below; see the re-entrancy note above.
    let e0599_method_bounds =
        diag.code == Some(E0599) && main_message_text(diag).is_some_and(is_method_bounds_text);
    // A call to one of CGP's extensible-data operations can also fail before its bounds are
    // checked: the method probe finds no impl of the operation's trait at all (a builder
    // finalized too early has no `finalize_build`), or settles on the one impl left and rejects
    // the call's tag argument (a variant extracted twice is an `E0308`). Both are reported while
    // the calling body is type-checked, as the method-bounds `E0599` is, so the solver may run,
    // and only the data-call anchor takes them.
    let data_call_only = is_no_matching_item(diag) || diag.code == Some(E0308);
    // Only a diagnostic reporting an unmet obligation has a chain to walk: a projection or
    // trait-bound failure, or a method whose bounds fail. A coherence conflict (`E0119`) names
    // CGP traits too, but two impls overlapping is not a bound left unmet, so walking it would
    // invent a missing-impl cause.
    if !matches!(diag.code, Some(E0271) | Some(E0277)) && !e0599_method_bounds && !data_call_only {
        return None;
    }
    // A `?`-operator error is about `Try` on the call's result, not about the call's bounds; the
    // cascade gate in `emit_diagnostic` drops one that follows a CGP failure, and any other is
    // left as rustc wrote it rather than re-anchored onto the call it overlaps.
    if is_question_mark_cascade(diag) {
        return None;
    }
    let primary_span = diag.span.primary_span()?;
    if data_call_only {
        let resolved = resolve::resolve_data_call(cx?, diag.span.primary_spans())?;
        // A mismatched argument is a plain type error unless the walk finds the field or
        // variant state the operation rejects behind it.
        if diag.code == Some(E0308)
            && !resolved
                .causes
                .iter()
                .all(|cause| cause.leaf.is_data_state())
        {
            return None;
        }
        return Some((resolved, primary_span, true));
    }
    let (resolved, at_call) = anchor(cx?, diag)?;
    Some((resolved, primary_span, at_call))
}

/// The anchors `try_resolve` tries for an unmet obligation, in order, with whether the resolution
/// is anchored at the call expression.
fn anchor(cx: ResolveCtx<'_, '_>, diag: &DiagInner) -> Option<(Resolved, bool)> {
    let primary_span = diag.span.primary_span()?;
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
    let resolved = resolve::resolve_check_failure(cx, primary_span)
        // A `#[check_providers]` entry is anchored the same way, on the provider it lists.
        .or_else(|| resolve::resolve_provider_check(cx, primary_span))
        .or_else(|| resolve::resolve_impl_site(cx, primary))
        .or_else(|| resolve::resolve_wrapper_chain(cx, primary))
        .or_else(|| resolve::resolve_use_site(cx, &spans))
        // A namespace-joined context's wiring lives in the namespace, not its own
        // `DelegateComponent` impls, so the per-component re-check above finds nothing;
        // anchoring on the consumer trait the diagnostic names and walking through the
        // namespace recovers it.
        .or_else(|| resolve::resolve_use_site_consumer(cx, &spans));
    if let Some(resolved) = resolved {
        return Some((resolved, false));
    }
    // The next resort re-reads the failing *call expression* itself — the anchor for a
    // consumer-method `E0277` whose spans never touch the context's definition (a
    // `Code`-dispatched handler pipeline that matches unconditionally), and for a direct
    // call to a `#[cgp_fn]` trait method. A resolution from here is flagged, so the
    // header is worded from the trait the call needs rather than from whichever provider
    // bound rustc's headline stopped on.
    if let Some(resolved) = resolve::resolve_call_site(cx, primary) {
        return Some((resolved, true));
    }
    // A call to one of CGP's extensible-data operations, whose value is often another data
    // operation's result (a builder chain, a remainder bound by a pattern) or named by a
    // type-relative path (`Shape::from_variant(…)`).
    if let Some(resolved) = resolve::resolve_data_call(cx, primary) {
        return Some((resolved, true));
    }
    // A provider trait's method called through the provider's own path
    // (`PipeHandlers::<…>::compute(&(), …)`), which asserts the provider trait directly.
    if let Some(resolved) = resolve::resolve_provider_call(cx, primary) {
        return Some((resolved, true));
    }
    // A bound of a generic function the call instantiates (`fail::<App>()`), read off the
    // function's own `where` clauses: an `E0277` whose trait no span names and whose
    // context no receiver carries.
    if diag.code == Some(E0277)
        && let Some(resolved) = resolve::resolve_bound_call(cx, primary)
    {
        return Some((resolved, true));
    }
    // A `#[cgp_fn]` / `#[blanket_trait]` blanket trait the diagnostic names in its spans,
    // required through a `where` bound or supertrait rather than a direct call. This is
    // gated to `E0277` — a trait *used as a bound* — deliberately: an `E0599` method call
    // is the call-site anchor's domain, and a *generic consumer* method call whose deep
    // trait bound is a note (not the failure the diagnostic is about) must stay declined
    // when its dispatch parameter is unrecoverable, rather than latch onto that transitive
    // trait (see `generic_consumer_unwritten_arg`).
    if diag.code == Some(E0277)
        && let Some(resolved) = resolve::resolve_use_site_blanket_trait(cx, &spans)
    {
        return Some((resolved, false));
    }
    // A `Key -> Table` wiring entry whose table lacks the key: the compiler reports the
    // entry's own forwarding bound, and the consumer the entry wires names the cause.
    if diag.code == Some(E0277)
        && let Some(resolved) = resolve::resolve_forwarding_entry(cx, primary)
    {
        return Some((resolved, true));
    }
    None
}

/// Transform a resolved wiring failure in place from the rustc-free [`plan_resolved`]: replace
/// the main message when the plan carries a coded header (re-aiming the caret at the failing
/// entry), then replace the sub-messages with the plan's fix `help`s, dropping rustc's own
/// suggestions; the `root cause:` note is returned for the flush to render. A resolution
/// anchored at the call expression (`at_call`) plans as a use-site failure whatever its rustc
/// code, so its class is the consumer the call needs, except a genuine field-type mismatch,
/// whose `E0271` class is the more specific `[CGP-E004]`.
pub fn transform_resolved(
    diag: &mut DiagInner,
    resolved: &Resolved,
    span: Span,
    at_call: bool,
    names: &ComponentNameMap,
) -> TransformedResolution {
    let kind = match diag_kind(diag) {
        kind if at_call && kind != DiagKind::TypeMismatch => DiagKind::MethodNotFound,
        kind => kind,
    };
    let plan = plan_resolved(kind, main_message_text(diag), resolved, names);
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

/// Why the `E0275` overflow `diag` never resolves, when walking the wiring of its check entry, or
/// of the consumer-method call at its caret, shows it: a cycle, or a lookup that grows
/// ([`resolve::overflow_walk`]). `None` for any other diagnostic, and for an overflow the walk
/// does not explain (one that follows from an unresolved type, say), which keeps rustc's
/// wording.
pub fn wiring_overflow(cx: Option<ResolveCtx<'_, '_>>, diag: &DiagInner) -> Option<WiringOverflow> {
    if diag.code != Some(E0275) {
        return None;
    }
    let span = diag.span.primary_span()?;
    let requirement = main_message_text(diag)?
        .strip_prefix("overflow evaluating the requirement `")?
        .split_once('`')?
        .0;
    match resolve::overflow_walk(cx?, span, requirement) {
        resolve::OverflowWalk::Cut(reason) => Some(reason),
        resolve::OverflowWalk::NotCut | resolve::OverflowWalk::NoEntry => None,
    }
}

/// Rewrite every recognized CGP wiring message in `diag`, in place: the first fallback text pass
/// for a diagnostic the typed resolver declined. The primary header takes the full rewrite
/// (including the coded main-message forms); the children take only the obligation-chain rename,
/// since a CGP error code belongs on a main message and never on a sub-message. A message that is
/// not a wiring form is left untouched, and the name map is forced only when some message is
/// actually rewritten.
pub fn rewrite(diag: &mut DiagInner, names: &ComponentNameMap) -> bool {
    let mut changed = rewrite_messages(&mut diag.messages, names, rewrite_message);
    for child in &mut diag.children {
        changed |= rewrite_messages(&mut child.messages, names, rewrite_required_for);
    }
    changed
}
