//! The anchors as one ordered list: which diagnostic each may read, and what its resolution means.
//!
//! Each anchor recovers the obligation to walk from one failure shape, and the first that succeeds
//! wins, so the order is part of the behaviour. See
//! `cgp-knowledge-base/cargo-cgp/implementation/typed-resolution-anchors.md`.

use cargo_cgp_error_processing::Resolved;
use rustc_errors::ErrCode;
use rustc_errors::codes::{E0277, E0308};
use rustc_span::Span;

use crate::resolve::{
    ResolveCtx, resolve_bound_call, resolve_call_site, resolve_check_failure, resolve_data_call,
    resolve_forwarding_entry, resolve_impl_site, resolve_provider_call, resolve_provider_check,
    resolve_use_site, resolve_use_site_blanket_trait, resolve_use_site_consumer,
    resolve_wrapper_chain,
};

/// What the anchors read from a diagnostic.
pub struct AnchorInput<'d> {
    /// The diagnostic's rustc code.
    pub code: Option<ErrCode>,
    /// The first primary span, the caret a check entry matches.
    pub primary_span: Span,
    /// Every primary span. An anchor that re-anchors a diagnostic onto code it sits inside reads
    /// these alone: a secondary label or a note can point at any code the failure passed through.
    pub primary_spans: &'d [Span],
    /// Every span of the diagnostic, labels and notes included.
    pub spans: Vec<Span>,
    /// Whether the diagnostic is a shape only the data-call anchor reads: a "no method named"
    /// `E0599`, or an `E0308` on a call's argument.
    pub data_call_only: bool,
}

/// One entry of the anchor list.
pub struct Anchor {
    /// Whether the anchor may read this diagnostic.
    pub gate: fn(&AnchorInput<'_>) -> bool,
    /// The anchor itself, given the part of the input it reads.
    pub run: fn(ResolveCtx<'_, '_>, &AnchorInput<'_>) -> Option<Resolved>,
    /// Whether the diagnostic is what this resolution explains.
    pub accept: fn(&AnchorInput<'_>, &Resolved) -> bool,
    /// Whether the resolution is anchored at the call expression, so it plans as a use-site failure
    /// whatever its rustc code.
    pub at_call: bool,
}

/// A diagnostic reporting an unmet obligation, which every shape but the data-call one is.
fn unmet_obligation(input: &AnchorInput<'_>) -> bool {
    !input.data_call_only
}

/// A trait used as a bound, rather than a method call or a projection.
fn trait_bound(input: &AnchorInput<'_>) -> bool {
    input.code == Some(E0277)
}

/// Any resolution explains its diagnostic.
fn accept_any(_: &AnchorInput<'_>, _: &Resolved) -> bool {
    true
}

/// A mismatched argument is a plain type error unless the walk finds the field or variant state the
/// operation rejects behind it.
fn accept_data_call(input: &AnchorInput<'_>, resolved: &Resolved) -> bool {
    input.code != Some(E0308)
        || resolved
            .causes
            .iter()
            .all(|cause| cause.leaf.is_data_state())
}

/// The anchors, in the order they are tried.
pub const ANCHORS: [Anchor; 12] = [
    // A check entry: the obligation recovered from the check impl at the caret.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_check_failure(cx, input.primary_span),
        accept: accept_any,
        at_call: false,
    },
    // A `#[check_providers]` entry, anchored the same way on the provider it lists.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_provider_check(cx, input.primary_span),
        accept: accept_any,
        at_call: false,
    },
    // A hand-written `impl … for Context` block the failure surfaces inside, which recovers the exact
    // failing obligation with its concrete component parameters, more precisely than a use-site
    // re-check.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_impl_site(cx, input.primary_spans),
        accept: accept_any,
        at_call: false,
    },
    // A wrapper implemented on a foreign type holding the context (`impl … for Router<Arc<MockApp>>`),
    // whose CGP consumer failure sits several `where`-clause hops down.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_wrapper_chain(cx, input.primary_spans),
        accept: accept_any,
        at_call: false,
    },
    // A use-site failure such as a consumer-method call, whose obligation no check impl carries:
    // the context comes from the diagnostic's spans.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_use_site(cx, &input.spans),
        accept: accept_any,
        at_call: false,
    },
    // A namespace-joined context's wiring lives in the namespace, not its own `DelegateComponent`
    // impls, so the re-check above finds nothing; the consumer trait the diagnostic names leads
    // through the namespace.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_use_site_consumer(cx, &input.spans),
        accept: accept_any,
        at_call: false,
    },
    // The failing call expression itself: a consumer-method `E0277` whose spans never touch the
    // context's definition (a `Code`-dispatched handler pipeline that matches unconditionally), and
    // a direct call to a `#[cgp_fn]` trait method.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_call_site(cx, input.primary_spans),
        accept: accept_any,
        at_call: true,
    },
    // A call to one of CGP's extensible-data operations, whose value is often another data
    // operation's result (a builder chain, a remainder bound by a pattern) or named by a
    // type-relative path (`Shape::from_variant(…)`). The one entry a data-call-only shape reaches.
    Anchor {
        gate: |_| true,
        run: |cx, input| resolve_data_call(cx, input.primary_spans),
        accept: accept_data_call,
        at_call: true,
    },
    // A provider trait's method called through the provider's own path
    // (`PipeHandlers::<…>::compute(&(), …)`), which asserts the provider trait directly.
    Anchor {
        gate: unmet_obligation,
        run: |cx, input| resolve_provider_call(cx, input.primary_spans),
        accept: accept_any,
        at_call: true,
    },
    // A bound of a generic function the call instantiates (`fail::<App>()`), read off the function's
    // own `where` clauses: an `E0277` whose trait no span names and whose context no receiver carries.
    Anchor {
        gate: trait_bound,
        run: |cx, input| resolve_bound_call(cx, input.primary_spans),
        accept: accept_any,
        at_call: true,
    },
    // A `#[cgp_fn]` / `#[blanket_trait]` blanket trait the diagnostic names in its spans, required
    // through a `where` bound or supertrait rather than a direct call. The gate is deliberate: an
    // `E0599` method call is the call-site anchor's, and a generic consumer's method call whose deep
    // trait bound is only a note must stay declined when its dispatch parameter is unrecoverable,
    // rather than latch onto that transitive trait (see `generic_consumer_unwritten_arg`).
    Anchor {
        gate: trait_bound,
        run: |cx, input| resolve_use_site_blanket_trait(cx, &input.spans),
        accept: accept_any,
        at_call: false,
    },
    // A `Key -> Table` wiring entry whose table lacks the key: the compiler reports the entry's own
    // forwarding bound, and the consumer the entry wires names the cause.
    Anchor {
        gate: trait_bound,
        run: |cx, input| resolve_forwarding_entry(cx, input.primary_spans),
        accept: accept_any,
        at_call: true,
    },
];

/// Try the anchors in order, and return the first resolution an entry accepts, with whether it is
/// anchored at the call expression.
pub fn resolve(cx: ResolveCtx<'_, '_>, input: &AnchorInput<'_>) -> Option<(Resolved, bool)> {
    ANCHORS
        .iter()
        .filter(|anchor| (anchor.gate)(input))
        .find_map(|anchor| {
            let resolved = (anchor.run)(cx, input)?;
            (anchor.accept)(input, &resolved).then_some((resolved, anchor.at_call))
        })
}
