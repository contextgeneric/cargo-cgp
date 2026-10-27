//! Locating the failing call and the consumer traits it could resolve through.

use cargo_cgp_error_processing::Resolved;
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind, HirId};
use rustc_middle::ty::{self, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::resolve::cache::ResolveCache;
use crate::resolve::call_site::{CallForm, receiver_context, seed_from_call, unbounded_receiver};
use crate::resolve::cgp_item::{
    carries_wiring, is_blanket_trait, is_consumer_trait, is_context_adt,
};
use crate::resolve::walk::{holds, resolve_leaves};

/// Resolve a use-site failure by re-reading the failing *call expression*: recover the context
/// from the receiver's binding, the component's parameters by unifying the call's written
/// argument types against the method's declared signature, and seed the walk with a rigid
/// placeholder for each parameter the call leaves to inference. `None` when no method call sits
/// at the diagnostic's primary spans, the receiver's type is not syntactically recoverable, or no
/// placeholder-free root cause is found.
///
/// The called method resolves through either a CGP **consumer trait** or a `#[cgp_fn]` /
/// `#[blanket_trait]` **blanket trait** (a blanket-impl trait, in any crate, that is not a CGP
/// component and whose blanket depends on CGP). Both are consumed the same way —
/// `app.describe()` — and both seed a walkable obligation `Ctx: Trait<…>` whose `Self` is the
/// context; they differ only in the header the result gets. A blanket trait is not a CGP
/// component, so — exactly as the impl-site anchor words such a trait — its failure reads
/// `[CGP-E003] the trait …` rather than `[CGP-E001] the consumer trait …`, by clearing
/// [`Resolved::consumers_are_cgp`] the walk sets.
///
/// Tried after the six span-matching anchors, so a failure any of them can recover keeps its more
/// precise recovery.
pub fn resolve_call_site(
    tcx: TyCtxt<'_>,
    cache: &ResolveCache,
    spans: &[Span],
) -> Option<Resolved> {
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
        // context, and a generic one's arguments are exactly what the missing typeck results would
        // have supplied.
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
            if let Some(mut resolved) = resolve_leaves(tcx, cache, top) {
                // A `#[cgp_fn]` / `#[blanket_trait]` blanket trait is consumed like a consumer trait
                // but is not a CGP *component*, so it heads the diagnostic as `[CGP-E003] the trait
                // …` — the same wording the impl-site anchor gives such a trait reached through a
                // wrapper. A genuine consumer keeps the `consumers_are_cgp` the walk set. The trait
                // is called on a type that is a context only when it carries wiring, so a type with
                // none is named plainly.
                if kind == CallTrait::Blanket {
                    resolved.consumers_are_cgp = false;
                    resolved.subject_is_context = carries_wiring(tcx, context);
                }
                return Some(resolved);
            }
        }
    }
    None
}

/// One method call found at the diagnostic's primary spans: the method name, the receiver expression,
/// the call's argument expressions, and the call expression's id, for which the compiler records
/// the traits in scope.
pub(crate) struct MethodCall<'tcx> {
    pub(crate) method: Symbol,
    pub(crate) receiver: &'tcx Expr<'tcx>,
    pub(crate) args: &'tcx [Expr<'tcx>],
    pub(crate) hir_id: HirId,
}

/// Every method-call expression in a local body that one of the diagnostic's primary spans is about.
/// A use-site failure's primary span sits on the method name, on one whole argument, or on the
/// whole call (rustc's re-report where the result is awaited included), so a call is collected when
/// a span overlaps it without lying strictly inside its receiver or one of its arguments. An error
/// inside an argument (an `Rc` passed to a `Send` bound in `person.greet(count(Rc::new(1)))`) is
/// about that argument's own code, and an error elsewhere in the enclosing body is about no call.
pub(crate) fn method_calls_at<'tcx>(tcx: TyCtxt<'tcx>, spans: &[Span]) -> Vec<MethodCall<'tcx>> {
    let mut finder = CallFinder {
        spans,
        calls: Vec::new(),
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.calls
}

struct CallFinder<'a, 'tcx> {
    spans: &'a [Span],
    calls: Vec<MethodCall<'tcx>>,
}

impl<'tcx> Visitor<'tcx> for CallFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::MethodCall(segment, receiver, args, _) = expr.kind
            && self
                .spans
                .iter()
                .any(|&span| is_about_call(span, expr.span, std::iter::once(receiver).chain(args)))
        {
            self.calls.push(MethodCall {
                method: segment.ident.name,
                receiver,
                args,
                hir_id: expr.hir_id,
            });
        }
        intravisit::walk_expr(self, expr);
    }
}

/// Whether a diagnostic's primary `span` is about the call spanning `call`: it overlaps the call
/// and does not lie strictly inside one of its `operands` (a method call's receiver and arguments,
/// or a path call's arguments). A span equal to a whole argument (the `PhantomData` tag a failing
/// handler call is reported on) is about the call.
pub(crate) fn is_about_call<'a, 'tcx: 'a>(
    span: Span,
    call: Span,
    operands: impl IntoIterator<Item = &'a Expr<'tcx>>,
) -> bool {
    span.overlaps(call)
        && !operands
            .into_iter()
            .any(|operand| operand.span.contains(span) && operand.span != span)
}

/// What kind of trait a call resolves through, which decides the class its failure is headed with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CallTrait {
    /// A CGP consumer trait (`[CGP-E001]`).
    Consumer,
    /// A `#[cgp_fn]` / `#[blanket_trait]` blanket trait (`[CGP-E003]`).
    Blanket,
}

/// Every trait declaring a `self`-receiver associated fn named `method` that a call by that name
/// could resolve through — the candidates [`resolve_call_site`] tries — each paired with that
/// method's `DefId` (whose declared signature [`seed_from_call`] unifies the
/// call's arguments against) and the [kind](CallTrait) of trait it is.
///
/// Two kinds qualify, and CGP consumer traits come **first** so a directly-wired consumer keeps
/// its precise `[CGP-E001]` recovery before any other trait is tried:
///
/// - a **CGP consumer trait** (recognized structurally, in any crate); and
/// - a **blanket-impl trait** that is not a consumer and shows CGP evidence — the shape `#[cgp_fn]`
///   / `#[blanket_trait]` generate (`impl<Context> Describe for Context where Self: …`). Its
///   obligation `Ctx: Describe` is walkable exactly as a consumer's is, since its `Self` is the
///   context.
///
/// CGP's extensible-data traits are left to the [data-call anchor](super::resolve_data_call),
/// which types a receiver through a chain of data operations.
///
/// Requiring CGP evidence of the second kind excludes ordinary blanket traits (`Into`, `From`); the
/// `self`-method requirement and the failing-obligation gate in [`resolve_call_site`] keep a wrong
/// guess from fabricating a diagnostic.
pub(crate) fn traits_with_method(
    tcx: TyCtxt<'_>,
    method: Symbol,
) -> Vec<(DefId, DefId, CallTrait)> {
    let mut found: Vec<(DefId, DefId, CallTrait)> = Vec::new();
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
            found.push((trait_did, method_did, CallTrait::Consumer));
        } else if is_blanket_trait(tcx, trait_did) {
            found.push((trait_did, method_did, CallTrait::Blanket));
        }
    }
    // A stable sort keeps each kind in the order the traits were listed.
    found.sort_by_key(|&(_, _, kind)| kind as u8);
    found
}
