//! Locating the failing call and the consumer traits it could resolve through.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind};
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::resolve::cache::ResolveCache;
use crate::resolve::call_site::{receiver_context, seed_from_call};
use crate::resolve::cgp_item::{is_consumer_trait, is_local_adt, is_local_blanket_trait};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve a use-site failure by re-reading the failing *call expression*: recover the context
/// from the receiver's binding, the component's parameters by unifying the call's written
/// argument types against the method's declared signature, and seed the walk with a rigid
/// placeholder for each parameter the call leaves to inference. `None` when no method call sits
/// at the diagnostic's spans, the receiver's type is not syntactically recoverable, or no
/// placeholder-free root cause is found.
///
/// The called method resolves through either a CGP **consumer trait** or a `#[cgp_fn]` /
/// `#[blanket_trait]` **capability trait** (a local blanket-impl trait that is not a CGP
/// component). Both are consumed the same way — `app.describe()` — and both seed a walkable
/// obligation `Ctx: Trait<…>` whose `Self` is the context; they differ only in the header the
/// result gets. A capability trait is not a CGP component, so — exactly as the impl-site anchor
/// words such a trait — its failure reads `[CGP-E009] the trait …` rather than `[CGP-E001] the
/// consumer trait …`, by clearing [`Resolved::consumers_are_cgp`] the walk sets.
///
/// Tried last: a failure any span-matching anchor can recover keeps its more precise recovery.
pub fn resolve_call_site(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
) -> Option<Resolved> {
    for call in method_calls_at(tcx, spans) {
        let Some(context) = receiver_context(tcx, call.receiver) else {
            continue;
        };
        // Only a local, monomorphic ADT can be re-checked as a context: a foreign receiver is not
        // a CGP context of this crate, and a generic one's arguments are exactly what the missing
        // typeck results would have supplied.
        if !is_local_adt(context) || context.has_param() {
            continue;
        }
        for (trait_did, method_did, is_cgp_consumer) in traits_with_method(tcx, call.method) {
            let Some(top) = seed_from_call(tcx, context, trait_did, method_did, call.args) else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(mut resolved) = resolve_leaves(tcx, cache, top) {
                // A `#[cgp_fn]` / `#[blanket_trait]` capability is consumed like a consumer trait
                // but is not a CGP *component*, so it heads the diagnostic as `[CGP-E009] the trait
                // …` — the same wording the impl-site anchor gives such a trait reached through a
                // wrapper. A genuine consumer keeps the `consumers_are_cgp` the walk set.
                if !is_cgp_consumer {
                    resolved.consumers_are_cgp = false;
                }
                return Some(resolved);
            }
        }
    }
    None
}

/// One method call found at the diagnostic's spans: the method name, the receiver expression,
/// and the call's argument expressions.
pub(crate) struct MethodCall<'tcx> {
    pub(crate) method: Symbol,
    pub(crate) receiver: &'tcx Expr<'tcx>,
    pub(crate) args: &'tcx [Expr<'tcx>],
}

/// Every method-call expression in a local body at one of the diagnostic's spans. A use-site
/// failure's spans sit on the method name, an argument, the whole call — or, for the re-report
/// rustc raises where the result is awaited, on the `.await` alone, whose desugared wrapper
/// expressions contain the call without the call's own span overlapping. So a method call is
/// collected when its own span overlaps a diagnostic span *or* it sits inside any expression
/// whose span does; each match is a candidate the caller tries (and gates on actually failing).
pub(crate) fn method_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<MethodCall<'tcx>> {
    let mut finder = CallFinder {
        spans,
        within_match: false,
        calls: Vec::new(),
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.calls
}

struct CallFinder<'a, 'tcx> {
    spans: &'a [Span],
    /// Whether the current expression sits inside one whose span overlapped a diagnostic span.
    within_match: bool,
    calls: Vec<MethodCall<'tcx>>,
}

impl<'tcx> Visitor<'tcx> for CallFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        let matched = self.within_match || self.spans.iter().any(|span| span.overlaps(expr.span));
        if matched && let ExprKind::MethodCall(segment, receiver, args, _) = expr.kind {
            self.calls.push(MethodCall {
                method: segment.ident.name,
                receiver,
                args,
            });
        }
        let saved = std::mem::replace(&mut self.within_match, matched);
        intravisit::walk_expr(self, expr);
        self.within_match = saved;
    }
}

/// Every trait declaring a `self`-receiver associated fn named `method` that a call by that name
/// could resolve through — the candidates [`resolve_call_site`] tries — each paired with that
/// method's `DefId` (whose declared signature [`seed_from_call`](super::seed_from_call) unifies the
/// call's arguments against) and whether the trait is a CGP *consumer* trait.
///
/// Two kinds qualify, and CGP consumer traits come **first** so a directly-wired consumer keeps its
/// precise `[CGP-E001]` recovery before any capability trait is tried:
///
/// - a **CGP consumer trait** (recognized structurally, in any crate); and
/// - a **local blanket-impl trait** that is not a consumer — the shape `#[cgp_fn]` /
///   `#[blanket_trait]` generate (`impl<Context> Describe for Context where Self: …`). Its
///   obligation `Ctx: Describe` is walkable exactly as a consumer's is, since its `Self` is the
///   context.
///
/// Restricting the second kind to *local* traits excludes foreign blanket traits (`Into`, `From`);
/// the `self`-method requirement and the failing-obligation gate in [`resolve_call_site`] keep a
/// wrong guess from fabricating a diagnostic.
pub(crate) fn traits_with_method(tcx: TyCtxt<'_>, method: Symbol) -> Vec<(DefId, DefId, bool)> {
    let mut consumers = Vec::new();
    let mut capabilities = Vec::new();
    for trait_did in tcx.all_traits_including_private() {
        let Some(method_did) = tcx
            .associated_items(trait_did)
            .filter_by_name_unhygienic(method)
            .find(|item| matches!(item.kind, ty::AssocKind::Fn { has_self: true, .. }))
            .map(|item| item.def_id)
        else {
            continue;
        };
        if is_consumer_trait(tcx, trait_did) {
            consumers.push((trait_did, method_did, true));
        } else if is_local_blanket_trait(tcx, trait_did) {
            capabilities.push((trait_did, method_did, false));
        }
    }
    consumers.into_iter().chain(capabilities).collect()
}
