//! Anchoring a failing call to one of CGP's extensible-data operations.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind, HirId, QPath};
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::{Span, Symbol};

use crate::resolve::call_site::{
    CallForm, data_expr_ty, data_fns_named, is_about_call, lower_hir_ty, peel_hir_refs,
    seed_from_call,
};
use crate::resolve::cgp_item::carries_wiring;
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve a failing call to one of CGP's extensible-data operations (a builder, an extractor, a
/// cast), a method call (`builder.build_field(…)`, `remainder.extract_field(…)`) or a type-relative
/// one (`Shape::from_variant(…)`). The value the operation is called on is typed through the chain
/// of data operations that produced it (`data_expr_ty`), so a builder chain or a remainder bound
/// by a pattern is read without typeck; the call is then seeded and walked like any call-site
/// failure. Only a data trait in scope at the call is a candidate, as the compiler's own lookup
/// considers only those, so a method of another library that shares a name (`builder`, `set`) is
/// never read as a data operation. The result heads the diagnostic as `[CGP-E050]`. `None` when no
/// data call sits at the diagnostic's primary spans, its subject cannot be typed, or no root cause
/// is found.
pub fn resolve_data_call(cx: ResolveCtx<'_, '_>, spans: &[Span]) -> Option<Resolved> {
    let tcx = cx.tcx;
    for call in data_calls_at(tcx, spans) {
        let Some(subject) = call.subject else {
            continue;
        };
        let method_call = call.form == CallForm::DataMethod;
        let in_scope = tcx.in_scope_traits(call.scope).unwrap_or_default();
        let candidates = data_fns_named(tcx, call.name, method_call)
            .into_iter()
            .filter(|&(trait_did, _)| {
                in_scope
                    .iter()
                    .any(|candidate| candidate.def_id == trait_did)
            });
        for (trait_did, fn_did) in candidates {
            let Some(top) = seed_from_call(tcx, subject, trait_did, fn_did, call.args, call.form)
            else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(mut resolved) = resolve_leaves(cx, top) {
                // A data trait is not a CGP component, and the value it is called on is a context
                // only when it carries wiring.
                resolved.consumers_are_cgp = false;
                resolved.data_operation = true;
                resolved.subject_is_context = carries_wiring(tcx, subject);
                return Some(resolved);
            }
        }
    }
    None
}

/// One call found at the diagnostic's primary spans that could be a data operation: the operation's
/// name, the type of the value it is called on (when it can be read), the call's arguments, how it
/// passes them, and the expression the compiler records the traits in scope for (the method call,
/// or a path call's callee).
struct DataCall<'tcx> {
    name: Symbol,
    subject: Option<Ty<'tcx>>,
    args: &'tcx [Expr<'tcx>],
    form: CallForm,
    scope: HirId,
}

/// Every method call, and every call by a type-relative path (`T::f(…)`), in a local body that one
/// of the diagnostic's primary spans is about, by the rule the call-site anchor applies
/// ([`is_about_call`]), outermost first.
fn data_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<DataCall<'tcx>> {
    let mut finder = DataCallFinder {
        tcx,
        spans,
        calls: Vec::new(),
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.calls
}

struct DataCallFinder<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    spans: &'a [Span],
    calls: Vec<DataCall<'tcx>>,
}

impl<'tcx> Visitor<'tcx> for DataCallFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        let call = match expr.kind {
            ExprKind::MethodCall(segment, receiver, args, _)
                if self.is_about(expr, std::iter::once(receiver).chain(args)) =>
            {
                Some(DataCall {
                    name: segment.ident.name,
                    subject: data_expr_ty(self.tcx, receiver),
                    args,
                    form: CallForm::DataMethod,
                    scope: expr.hir_id,
                })
            }
            ExprKind::Call(callee, args) => match callee.kind {
                ExprKind::Path(QPath::TypeRelative(qself, segment))
                    if self.is_about(expr, args) =>
                {
                    Some(DataCall {
                        name: segment.ident.name,
                        subject: lower_hir_ty(self.tcx, peel_hir_refs(qself)),
                        args,
                        form: CallForm::Path,
                        scope: callee.hir_id,
                    })
                }
                _ => None,
            },
            _ => None,
        };
        self.calls.extend(call);
        intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> DataCallFinder<'_, 'tcx> {
    /// Whether one of the primary spans is about the call `expr`, whose operands are `operands`.
    fn is_about<'a>(
        &self,
        expr: &Expr<'tcx>,
        operands: impl IntoIterator<Item = &'a Expr<'tcx>> + Clone,
    ) -> bool
    where
        'tcx: 'a,
    {
        self.spans
            .iter()
            .any(|&span| is_about_call(span, expr.span, operands.clone()))
    }
}
