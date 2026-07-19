//! Locating the failing call and the consumer traits it could resolve through.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind};
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::resolve::call_site::{receiver_context, seed_from_call};
use crate::resolve::cgp_item::{is_consumer_trait, is_local_adt};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve a use-site failure by re-reading the failing *call expression*: recover the context
/// from the receiver's binding, the component's parameters by unifying the call's written
/// argument types against the method's declared signature, and seed the walk with a rigid
/// placeholder for each parameter the call leaves to inference. `None` when no method call sits
/// at the diagnostic's spans, the receiver's type is not syntactically recoverable, or no
/// placeholder-free root cause is found.
///
/// Tried last: a failure any span-matching anchor can recover keeps its more precise recovery.
pub fn resolve_call_site(tcx: TyCtxt<'_>, spans: &[Span]) -> Option<Resolved> {
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
        for (consumer_did, method_did) in consumer_traits_with_method(tcx, call.method) {
            let Some(top) = seed_from_call(tcx, context, consumer_did, method_did, call.args)
            else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(resolved) = resolve_leaves(tcx, top) {
                return Some(resolved);
            }
        }
    }
    None
}

/// One method call found at the diagnostic's spans: the method name, the receiver expression,
/// and the call's argument expressions.
struct MethodCall<'tcx> {
    method: Symbol,
    receiver: &'tcx Expr<'tcx>,
    args: &'tcx [Expr<'tcx>],
}

/// Every method-call expression in a local body at one of the diagnostic's spans. A use-site
/// failure's spans sit on the method name, an argument, the whole call — or, for the re-report
/// rustc raises where the result is awaited, on the `.await` alone, whose desugared wrapper
/// expressions contain the call without the call's own span overlapping. So a method call is
/// collected when its own span overlaps a diagnostic span *or* it sits inside any expression
/// whose span does; each match is a candidate the caller tries (and gates on actually failing).
fn method_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<MethodCall<'tcx>> {
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

/// Every CGP consumer trait (recognized structurally, cross-crate) declaring a `self`-receiver
/// associated fn named `method` — the candidates a method call by that name can resolve through —
/// paired with that method's `DefId`, whose declared signature
/// [`seed_from_call`](super::seed_from_call) unifies the call's arguments against.
fn consumer_traits_with_method(tcx: TyCtxt<'_>, method: Symbol) -> Vec<(DefId, DefId)> {
    tcx.all_traits_including_private()
        .filter_map(|trait_did| {
            let method_did = tcx
                .associated_items(trait_did)
                .filter_by_name_unhygienic(method)
                .find(|item| matches!(item.kind, ty::AssocKind::Fn { has_self: true, .. }))?
                .def_id;
            Some((trait_did, method_did))
        })
        .filter(|&(trait_did, _)| is_consumer_trait(tcx, trait_did))
        .collect()
}
