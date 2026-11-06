//! Locating the failing call and the consumer traits it could resolve through.

use cargo_cgp_error_processing::byte_range::{self as range, Operand};
use cargo_cgp_error_processing::{Resolved, ResolvedClass};
use rustc_hir::{Expr, ExprKind, HirId};
use rustc_middle::ty::{TyCtxt, TypeVisitableExt as _};
use rustc_span::{Span, Symbol};

use crate::resolve::call_site::{CallForm, receiver_context, seed_from_call, unbounded_receiver};
use crate::resolve::ctx::ResolveCtx;
use crate::resolve::query::{
    CallTrait, byte_range, carries_wiring, holds, is_context_adt, local_body_exprs,
    traits_with_method,
};
use crate::resolve::walk::resolve_leaves;

/// Resolve a use-site failure by re-reading the failing *call expression*: recover the context
/// from the receiver's binding, the component's parameters by unifying the call's written
/// argument types against the method's declared signature, and seed the walk with a rigid
/// placeholder for each parameter the call leaves to inference. `None` when no method call sits
/// at the diagnostic's primary spans, the receiver's type is not syntactically recoverable, or no
/// placeholder-free root cause is found.
///
/// The called method resolves through either a CGP **consumer trait** or a `#[cgp_fn]` /
/// `#[blanket_trait]` **blanket trait** (a blanket-impl trait, in any crate, that is not a CGP
/// component and whose blanket depends on CGP). Both are consumed the same way (
/// `app.describe()`), and both seed a walkable obligation `Ctx: Trait<…>` whose `Self` is the
/// context; they differ only in the header the result gets. A blanket trait is not a CGP
/// component, so, exactly as the impl-site anchor words such a trait, its failure reads
/// `[CGP-E003] the trait …` rather than `[CGP-E001] the consumer trait …`, by changing the
/// [`Resolved::class`] the walk sets.
///
/// Tried after the six span-matching anchors, so a failure any of them can recover keeps its more
/// precise recovery.
pub fn resolve_call_site(cx: ResolveCtx<'_, '_>, spans: &[Span]) -> Option<Resolved> {
    let tcx = cx.tcx;
    for call in method_calls_at(tcx, spans) {
        let Some(context) = receiver_context(tcx, call.receiver) else {
            // A receiver typed by a type parameter of the enclosing item is no context, but a
            // consumer method called on it fails for a bound the item does not declare.
            if let Some(resolved) = unbounded_receiver(tcx, &call) {
                return Some(resolved);
            }
            continue;
        };
        // Only a monomorphic context ADT can be re-checked: a type with no wiring of its own is no
        // context, and the missing typeck results would have supplied a generic one's arguments.
        if !is_context_adt(tcx, context) || context.has_param() {
            continue;
        }
        for (trait_did, method_did, kind) in traits_with_method(tcx, call.method) {
            let Some(top) = seed_from_call(
                tcx,
                context,
                trait_did,
                method_did,
                call.args,
                CallForm::Method,
            ) else {
                continue;
            };
            if holds(tcx, top) {
                continue;
            }
            if let Some(mut resolved) = resolve_leaves(cx, top) {
                // A `#[cgp_fn]` / `#[blanket_trait]` blanket trait is consumed like a consumer
                // trait but is not a CGP *component*, so it heads the diagnostic as `[CGP-E003] the
                // trait …`, the same wording the impl-site anchor gives such a trait reached
                // through a wrapper. A genuine consumer keeps the class the walk set. The trait is
                // called on a type that is a context only when it carries wiring, so a type with
                // none is named plainly.
                if kind == CallTrait::Blanket {
                    resolved.class = ResolvedClass::Wrapper;
                    resolved.subject_is_context = carries_wiring(tcx, context);
                }
                return Some(resolved);
            }
        }
    }
    None
}

/// One method call found at the diagnostic's primary spans: the method name, the receiver
/// expression, the call's argument expressions, and the call expression's id, for which the
/// compiler records the traits in scope.
pub struct MethodCall<'tcx> {
    /// The method's name as written.
    pub method: Symbol,
    /// The receiver expression.
    pub receiver: &'tcx Expr<'tcx>,
    /// The argument expressions, without the receiver.
    pub args: &'tcx [Expr<'tcx>],
    /// The call expression's id, for which the compiler records the traits in scope.
    pub hir_id: HirId,
}

/// Every method-call expression in a local body that one of the diagnostic's primary spans is
/// about. A use-site failure's primary span sits on the method name, on one whole argument, or on
/// the whole call (rustc's re-report where the result is awaited included), so a call is collected
/// when a span overlaps it without lying strictly inside its receiver or one of its arguments. An
/// error inside an argument (an `Rc` passed to a `Send` bound in `person.greet(count(Rc::new(1)))`)
/// is about that argument's own code, and an error elsewhere in the enclosing body is about no
/// call.
pub fn method_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<MethodCall<'tcx>> {
    local_body_exprs(tcx)
        .filter_map(|expr| {
            if let ExprKind::MethodCall(segment, receiver, args, _) = expr.kind
                && spans.iter().any(|&span| {
                    is_about_call(span, expr.span, std::iter::once(receiver).chain(args))
                })
            {
                Some(MethodCall {
                    method: segment.ident.name,
                    receiver,
                    args,
                    hir_id: expr.hir_id,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Whether a diagnostic's primary `span` is about the call spanning `call`: it overlaps the call
/// and does not lie strictly inside one of its `operands` (a method call's receiver and arguments,
/// or a path call's arguments). A span equal to a whole argument (the `PhantomData` tag a failing
/// handler call is reported on) is about the call.
pub fn is_about_call<'a, 'tcx: 'a>(
    span: Span,
    call: Span,
    operands: impl IntoIterator<Item = &'a Expr<'tcx>>,
) -> bool {
    range::is_about_call(
        byte_range(span),
        byte_range(call),
        operands.into_iter().map(|operand| Operand {
            range: byte_range(operand.span),
            is_caret: operand.span == span,
        }),
    )
}
