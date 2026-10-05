//! Detecting a CGP trait called in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared.
//!
//! `#[cgp_fn]` lowers a body into a blanket impl over a generated generic context
//! (`impl<__Context__> Describe for __Context__ where __Context__: GetName`). A trait the body
//! calls on `self` must be a `where` bound on that context — declared with `#[uses(…)]`. When the
//! body calls a trait the `#[uses]` list omits, the method cannot resolve and rustc reports a
//! vague `E0599` on `&__Context__` pointing at a transitive `HasField` bound.
//!
//! `#[cgp_impl]` lowers a provider the same way except that the impl is of the provider trait,
//! `impl<__Context__> Greeter<__Context__> for GreetHello`, with `self` renamed to a
//! `__context__: &__Context__` parameter. Its `Self` is the provider struct, so the context is the
//! provider trait's first argument instead, and the call is recognized by its receiver: a parameter
//! declared as that context. A call on any other parameter (`value: &Value`) is left alone, since
//! its fix is a bound on that parameter, which `#[uses]` cannot express.
//!
//! This module recognizes that shape off the compiler: the failing call sits inside a generated
//! impl over a bare type-parameter context and is made on that context, the call names a method of
//! a CGP trait, and that trait is *not* among the impl's `where` bounds. It fills the rustc-free
//! [`UndeclaredTrait`] the emitter words into a `[CGP-E012]` header and `#[uses(…)]` help.

use cargo_cgp_error_processing::UndeclaredTrait;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{Expr, ExprKind, HirId, Node, QPath};
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::def_id::DefId;
use rustc_span::{Span, Symbol};

use crate::resolve::call_site::traits_with_method;

/// Recognize an undeclared-trait failure and recover the trait to declare, or
/// `None` when the diagnostic is not one. Everything is keyed on `primary_span`, the failing method
/// call — never on the diagnostic's note spans (which point at *other* generated impls, such as the
/// called trait's own definition).
///
/// The three conditions, all read structurally off the compiler:
/// 1. the failing call sits inside a generated impl over the `__Context__` a `#[cgp_fn]` or
///    `#[cgp_impl]` generates, never a concrete context (the impl's `Self` for a `#[cgp_fn]` blanket
///    impl, the provider trait's first argument for a `#[cgp_impl]` provider impl), and is made on
///    that context: a receiver that is a parameter must be declared as the context, and a receiver
///    whose type no signature gives is accepted only in a `#[cgp_fn]` impl. The impl is found by
///    walking up from the call's own body owner (the generated method) to its parent impl, rather
///    than by span containment: a generated impl's item span does not reliably cover its body;
/// 2. the call names a method of a CGP trait (a consumer trait, or a
///    `#[cgp_fn]`/`#[blanket_trait]` blanket trait); and
/// 3. that trait is *not* already a `where` bound of the impl — so this is a genuinely omitted
///    dependency, not a trait whose own deeper wiring fails.
pub fn detect_undeclared_trait(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    spans: &[Span],
) -> Option<UndeclaredTrait> {
    let (method, call_hir_id, receiver) = failing_call_at(tcx, primary_span)?;

    // The impl enclosing the failing call. Walk up the def-parent chain from the call's innermost
    // body owner rather than taking a single parent: a plain `#[cgp_fn]` body owner *is* the
    // generated method (parent = impl), but an `#[async_trait]` body is an async-block coroutine
    // nested inside that method, so the impl is two hops up. Stop at the first enclosing trait impl.
    let owner = tcx.hir_enclosing_body_owner(call_hir_id);
    let impl_did = enclosing_trait_impl(tcx, owner.to_def_id())?;
    let trait_ref = tcx
        .impl_trait_ref(impl_did)
        .instantiate_identity()
        .skip_norm_wip();
    // The generic context the generated impl is over: a `#[cgp_fn]` blanket impl's `Self`, or a
    // `#[cgp_impl]` provider impl's first trait argument (`Greeter<__Context__> for GreetHello`).
    // A hand-written `impl … for ConcreteContext` has neither and is a different
    // (concrete-context) failure.
    let self_is_context = matches!(trait_ref.self_ty().kind(), ty::Param(_));
    let context = if self_is_context {
        trait_ref.self_ty()
    } else {
        // `args` begins with `Self`, so the provider trait's own first argument is the second.
        let first = trait_ref.args.types().nth(1)?;
        if !matches!(first.kind(), ty::Param(_)) {
            return None;
        }
        first
    };
    // The call must be on that context. A receiver that is a parameter of another type (a
    // `value: &Value` beside the context) needs a bound on `Value`, which `#[uses]` cannot give, so
    // it is not this failure. A receiver whose type cannot be read off a signature (a method call's
    // result) is accepted only in a `#[cgp_fn]` impl, whose `Self` is the context.
    match receiver.and_then(|binding| param_declared_type(tcx, binding)) {
        Some(declared) if declared != context => return None,
        Some(_) => {}
        None if self_is_context => {}
        None => return None,
    }

    // The CGP trait(s) declaring `method`. Several unrelated traits can share a method name
    // across modules (a `#[cgp_fn]` trait and a `#[cgp_component]` consumer both named
    // `fetch_storage_object`), so disambiguate to the one the diagnostic actually points at: the
    // "trait bound not satisfied" note spans the *failing* trait's own definition. Only if none is
    // referenced (a single unambiguous candidate) fall back to the full list.
    let candidates: Vec<DefId> = traits_with_method(tcx, method)
        .into_iter()
        .map(|(trait_did, _, _)| trait_did)
        .collect();
    let referenced: Vec<DefId> = candidates
        .iter()
        .copied()
        .filter(|&trait_did| is_referenced(tcx, trait_did, spans))
        .collect();
    let pool = if referenced.is_empty() {
        &candidates
    } else {
        &referenced
    };

    for &trait_did in pool {
        // Already a `where` bound of the impl → declared; the failure is something else.
        if impl_bounds_by(tcx, impl_did, trait_did) {
            continue;
        }
        return Some(UndeclaredTrait {
            trait_name: tcx.item_name(trait_did).to_string(),
        });
    }
    None
}

/// Whether the diagnostic's spans point into `trait_did`'s own definition — the "trait bound not
/// satisfied" note lands on the failing trait's `#[cgp_fn]`/`#[cgp_component]` definition,
/// which is how a same-named method on an unrelated trait in another module is told apart.
fn is_referenced(tcx: TyCtxt<'_>, trait_did: DefId, spans: &[Span]) -> bool {
    let def_span = tcx.def_span(trait_did);
    spans.iter().any(|&span| def_span.overlaps(span))
}

/// The failing method call the primary span points at: its name, its `HirId`, and, when its
/// receiver is a path to a local binding (`self`, or `__context__` in a provider), that binding's
/// `HirId`. The call is matched precisely on the method segment's own span, so a call nested beside
/// it in the same expression (a sibling argument of one `format!`, say) is not mistaken for the
/// failing one.
fn failing_call_at(tcx: TyCtxt<'_>, primary_span: Span) -> Option<(Symbol, HirId, Option<HirId>)> {
    struct Finder {
        span: Span,
        found: Option<(Symbol, HirId, Option<HirId>)>,
    }
    impl<'tcx> Visitor<'tcx> for Finder {
        fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
            if let ExprKind::MethodCall(segment, receiver, ..) = expr.kind
                && segment.ident.span.overlaps(self.span)
            {
                let binding = match receiver.kind {
                    ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
                        Res::Local(binding) => Some(binding),
                        _ => None,
                    },
                    _ => None,
                };
                self.found = Some((segment.ident.name, expr.hir_id, binding));
            }
            intravisit::walk_expr(self, expr);
        }
    }
    let mut finder = Finder {
        span: primary_span,
        found: None,
    };
    for owner in tcx.hir_body_owners() {
        finder.visit_expr(tcx.hir_body_owned_by(owner).value);
    }
    finder.found
}

/// The declared type, with references peeled, of the fn parameter `binding` names, read from the
/// enclosing function's signature rather than from typeck results, which do not exist yet for the
/// body whose type checking is reporting the error. `None` when `binding` is not a parameter. A
/// `#[cgp_impl]` provider's renamed `self`, declared `&__Context__`, is the case this exists for.
///
/// An `async fn` body sees its parameters through fresh bindings, since lowering moves each into the
/// future with an untyped `let __context__ = __context__;`, so such a forwarding `let` is followed
/// back to the parameter it copies.
fn param_declared_type<'tcx>(tcx: TyCtxt<'tcx>, mut binding: HirId) -> Option<Ty<'tcx>> {
    let param = loop {
        match tcx.parent_hir_node(binding) {
            Node::Param(param) => break param,
            Node::LetStmt(let_stmt) if let_stmt.ty.is_none() => {
                let Some(Expr {
                    kind: ExprKind::Path(QPath::Resolved(None, path)),
                    ..
                }) = let_stmt.init
                else {
                    return None;
                };
                let Res::Local(source) = path.res else {
                    return None;
                };
                if source == binding {
                    return None;
                }
                binding = source;
            }
            _ => return None,
        }
    };
    let owner = tcx.hir_enclosing_body_owner(binding);
    if !matches!(tcx.def_kind(owner), DefKind::AssocFn | DefKind::Fn) {
        return None;
    }
    let index = tcx
        .hir_body_owned_by(owner)
        .params
        .iter()
        .position(|candidate| candidate.pat.hir_id == param.pat.hir_id)?;
    let sig = tcx
        .fn_sig(owner.to_def_id())
        .instantiate_identity()
        .skip_norm_wip()
        .skip_binder();
    sig.inputs().get(index).map(|input| input.peel_refs())
}

/// The nearest enclosing trait impl of `did`, walking up the def-parent chain (through the
/// intervening `#[async_trait]` coroutine body and its method), or `None` if none is reached before
/// the crate root. This is what lets the detection work uniformly for a sync `#[cgp_fn]` body (impl
/// one hop up) and an async one (impl two hops up, past the async-block coroutine).
fn enclosing_trait_impl(tcx: TyCtxt<'_>, mut did: DefId) -> Option<DefId> {
    loop {
        if matches!(tcx.def_kind(did), DefKind::Impl { of_trait: true }) {
            return Some(did);
        }
        let parent = tcx.opt_parent(did)?;
        if parent == did {
            return None;
        }
        did = parent;
    }
}

/// Whether `impl_did`'s `where` clause carries a trait bound of trait `trait_did` — i.e. the impl
/// already declares that trait (via `#[uses]` or a hand-written bound).
fn impl_bounds_by(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId) -> bool {
    tcx.clauses_of(impl_did)
        .clauses
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|bound| bound.skip_binder().trait_ref.def_id == trait_did)
}
