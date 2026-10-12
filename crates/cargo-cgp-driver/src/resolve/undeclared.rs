//! Detecting a CGP trait called in a `#[cgp_fn]`/`#[cgp_impl]` body but not declared.
//!
//! `#[cgp_fn]` lowers a body into a blanket impl over a generated generic context
//! (`impl<__Context__> Describe for __Context__ where __Context__: GetName`). A trait the body
//! calls on `self`, or as an associated function on `Self`, must be a `where` bound on that
//! context, declared with `#[uses(…)]`. When the body calls a trait the `#[uses]` list omits, the
//! call cannot resolve and rustc reports a vague `E0599` on `&__Context__` pointing at a transitive
//! `HasField` bound, or, for `Self::method(…)`, no associated function found for the type parameter
//! `__Context__`, with suggestions spliced into the macro attribute.
//!
//! `#[cgp_impl]` lowers a provider the same way except that the impl is of the provider trait,
//! `impl<__Context__> Greeter<__Context__> for GreetHello`, with `self` renamed to a
//! `__context__: &__Context__` parameter and `Self` to `__Context__`. Its `Self` is the provider
//! struct, so the context is the provider trait's first argument instead, and the call is
//! recognized by its receiver: a parameter declared as that context, or the context named as the
//! qualifying type. A call on any other parameter (`value: &Value`) is left alone, since its fix is a
//! bound on that parameter, which `#[uses]` cannot express.
//!
//! This module recognizes that shape off the compiler: the failing call sits inside a generated
//! impl over a bare type-parameter context and is made on that context, the call names a method of
//! a CGP trait, and that trait is *not* among the impl's `where` bounds. It fills the rustc-free
//! [`UndeclaredTrait`] the emitter words into a `[CGP-E030]` header and `#[uses(…)]` help, with the
//! trait written with the arguments the call fixes.

use cargo_cgp_error_processing::UndeclaredTrait;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::intravisit::{self, Visitor};
use rustc_hir::{self as hir, Expr, ExprKind, HirId, Node, QPath};
use rustc_infer::infer::{BoundRegionConversionTime, InferCtxt, TyCtxtInferExt as _};
use rustc_middle::ty::print::{PrintTraitRefExt as _, with_forced_trimmed_paths};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span, Symbol};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::call_site::{expr_written_ty, lower_hir_ty, traits_with_method};
use crate::resolve::cgp_item::{is_blanket_trait, is_consumer_trait, is_provider_trait};

/// Recognize an undeclared-trait failure and recover the trait to declare, or
/// `None` when the diagnostic is not one. Everything is keyed on `primary_span`, the failing method
/// call, never on the diagnostic's note spans (which point at *other* generated impls, such as the
/// called trait's own definition).
///
/// The three conditions, all read structurally off the compiler:
/// 1. the failing call sits inside a generated impl over the `__Context__` a `#[cgp_fn]` or
///    `#[cgp_impl]` generates, never a concrete context (the impl's `Self` for a `#[cgp_fn]` blanket
///    impl, the provider trait's first argument for a `#[cgp_impl]` provider impl), and is made on
///    that context: a receiver that is a parameter must be declared as the context, a receiver
///    whose type no signature gives is accepted only in a `#[cgp_fn]` impl, and an associated
///    function's qualifying type must name the context. The impl is found by walking up from the
///    call's own body owner (the generated method) to its parent impl, rather than by span
///    containment: a generated impl's item span does not reliably cover its body;
/// 2. the call names a method of a CGP trait (a consumer trait, or a
///    `#[cgp_fn]`/`#[blanket_trait]` blanket trait); and
/// 3. that trait is *not* already a `where` bound of the impl, so this is a genuinely omitted
///    dependency, not a trait whose own deeper wiring fails.
pub fn detect_undeclared_trait(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    spans: &[Span],
) -> Option<UndeclaredTrait> {
    let call = failing_call_at(tcx, primary_span)?;

    // The impl enclosing the failing call. Walk up the def-parent chain from the call's innermost
    // body owner rather than taking a single parent: a plain `#[cgp_fn]` body owner *is* the
    // generated method (parent = impl), but an `#[async_trait]` body is an async-block coroutine
    // nested inside that method, so the impl is two hops up. Stop at the first enclosing trait impl.
    let owner = tcx.hir_enclosing_body_owner(call.hir_id);
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
    match call.receiver {
        CallReceiver::Value(binding) => {
            match binding.and_then(|binding| param_declared_input(tcx, binding)) {
                Some(declared) if declared.peel_refs() != context => return None,
                Some(_) => {}
                None if self_is_context => {}
                None => return None,
            }
        }
        CallReceiver::Type(qself) => {
            if !names_context(tcx, qself, context, self_is_context.then_some(impl_did)) {
                return None;
            }
        }
    }

    // The CGP trait(s) declaring `method`. Several unrelated traits can share a method name
    // across modules (a `#[cgp_fn]` trait and a `#[cgp_component]` consumer both named
    // `fetch_storage_object`), so disambiguate to the one the diagnostic actually points at: the
    // "trait bound not satisfied" note spans the *failing* trait's own definition. Only if none is
    // referenced (a single unambiguous candidate) fall back to the full list.
    let candidates: Vec<(DefId, DefId)> = match call.receiver {
        CallReceiver::Value(_) => traits_with_method(tcx, call.method)
            .into_iter()
            .map(|(trait_did, method_did, _)| (trait_did, method_did))
            .collect(),
        CallReceiver::Type(_) => traits_with_assoc_fn(tcx, call.method),
    };
    let referenced: Vec<(DefId, DefId)> = candidates
        .iter()
        .copied()
        .filter(|&(trait_did, _)| is_referenced(tcx, trait_did, spans))
        .collect();
    let pool = if referenced.is_empty() {
        &candidates
    } else {
        &referenced
    };

    for &(trait_did, method_did) in pool {
        // Already a `where` bound of the impl → declared; the failure is something else.
        if impl_bounds_by(tcx, impl_did, trait_did) {
            continue;
        }
        let has_self = tcx.associated_item(method_did).is_method();
        let (bound, unresolved) =
            written_bound(tcx, trait_did, method_did, context, call.args, has_self);
        return Some(UndeclaredTrait {
            trait_name: tcx.item_name(trait_did).to_string(),
            bound,
            unresolved,
        });
    }
    None
}

/// The failing call the primary span points at: the method's name, the call's `HirId`, what the
/// method is called on, and the call's argument expressions (the receiver excluded).
struct FailingCall<'tcx> {
    method: Symbol,
    hir_id: HirId,
    receiver: CallReceiver<'tcx>,
    args: &'tcx [Expr<'tcx>],
}

/// What a failing call is made on.
enum CallReceiver<'tcx> {
    /// A method call, `self.method(…)`: the local binding the receiver path names (`self`, or
    /// `__context__` in a provider), when it is one.
    Value(Option<HirId>),
    /// An associated-function call, `Self::method(…)`: the written qualifying type.
    Type(&'tcx hir::Ty<'tcx>),
}

/// Whether `trait_did`'s definition is among the diagnostic's spans. The "trait bound not
/// satisfied" note lands on the failing trait's `#[cgp_fn]`/`#[cgp_component]` definition,
/// which is how a same-named method on an unrelated trait in another module is told apart.
fn is_referenced(tcx: TyCtxt<'_>, trait_did: DefId, spans: &[Span]) -> bool {
    let def_span = tcx.def_span(trait_did);
    spans.iter().any(|&span| def_span.overlaps(span))
}

/// The failing call the primary span points at: a method call whose receiver is a path to a local
/// binding when it is one, or an associated-function call on a written type. The call is matched
/// precisely on the method segment's own span, so a call nested beside it in the same expression
/// (a sibling argument of one `format!`, say) is not mistaken for the failing one.
fn failing_call_at<'tcx>(tcx: TyCtxt<'tcx>, primary_span: Span) -> Option<FailingCall<'tcx>> {
    struct Finder<'tcx> {
        span: Span,
        found: Option<FailingCall<'tcx>>,
    }
    impl<'tcx> Visitor<'tcx> for Finder<'tcx> {
        fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
            match expr.kind {
                ExprKind::MethodCall(segment, receiver, args, _)
                    if segment.ident.span.overlaps(self.span) =>
                {
                    let binding = match receiver.kind {
                        ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
                            Res::Local(binding) => Some(binding),
                            _ => None,
                        },
                        _ => None,
                    };
                    self.found = Some(FailingCall {
                        method: segment.ident.name,
                        hir_id: expr.hir_id,
                        receiver: CallReceiver::Value(binding),
                        args,
                    });
                }
                ExprKind::Call(callee, args) => {
                    if let ExprKind::Path(QPath::TypeRelative(qself, segment)) = callee.kind
                        && segment.ident.span.overlaps(self.span)
                    {
                        self.found = Some(FailingCall {
                            method: segment.ident.name,
                            hir_id: expr.hir_id,
                            receiver: CallReceiver::Type(qself),
                            args,
                        });
                    }
                }
                _ => {}
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

/// Whether the written qualifying type of an associated-function call names the generated
/// context: a `#[cgp_impl]` body's `Self`, which the macro writes as the `__Context__` parameter, or
/// a `#[cgp_fn]` body's `Self`, the alias of a blanket impl whose `Self` is the context
/// (`blanket_impl`).
fn names_context(
    tcx: TyCtxt<'_>,
    qself: &hir::Ty<'_>,
    context: Ty<'_>,
    blanket_impl: Option<DefId>,
) -> bool {
    let hir::TyKind::Path(QPath::Resolved(None, path)) = qself.kind else {
        return false;
    };
    match path.res {
        Res::Def(DefKind::TyParam, param_did) => {
            matches!(context.kind(), ty::Param(param) if param.name == tcx.item_name(param_did))
        }
        Res::SelfTyAlias { alias_to, .. } => blanket_impl == Some(alias_to),
        _ => false,
    }
}

/// Every CGP trait declaring an associated function named `method`, with or without a `self`
/// receiver, each paired with that function's `DefId`: the candidates for a `Self::method(…)` call.
/// Consumer traits come first, then blanket traits that are not provider traits (a provider trait's
/// delegation blanket would otherwise pass as one, and a provider trait is never what `#[uses]`
/// declares).
fn traits_with_assoc_fn(tcx: TyCtxt<'_>, method: Symbol) -> Vec<(DefId, DefId)> {
    let mut consumers = Vec::new();
    let mut blanket_traits = Vec::new();
    for trait_did in tcx.all_traits_including_private() {
        let Some(method_did) = tcx
            .associated_items(trait_did)
            .filter_by_name_unhygienic(method)
            .find(|item| matches!(item.kind, ty::AssocKind::Fn { .. }))
            .map(|item| item.def_id)
        else {
            continue;
        };
        if is_consumer_trait(tcx, trait_did) {
            consumers.push((trait_did, method_did));
        } else if !is_provider_trait(tcx, trait_did) && is_blanket_trait(tcx, trait_did) {
            blanket_traits.push((trait_did, method_did));
        }
    }
    consumers.into_iter().chain(blanket_traits).collect()
}

/// The bound `#[uses(…)]` declares for `trait_did`, written with the arguments the call fixes, and
/// the trait's parameters it does not: each argument the call writes (or names through a parameter
/// or an annotated `let`) is unified with the method's declared input, as the call-site anchor
/// does, and a parameter that stays unknown, or comes out holding a lifetime or a generic parameter
/// a `#[uses]` bound could not write, is written as the trait's own parameter name for the reader to
/// replace. `(None, [])` when the trait takes a lifetime or const parameter.
fn written_bound<'tcx>(
    tcx: TyCtxt<'tcx>,
    trait_did: DefId,
    method_did: DefId,
    context: Ty<'tcx>,
    args: &[Expr<'tcx>],
    has_self: bool,
) -> (Option<String>, Vec<String>) {
    let generics = tcx.generics_of(trait_did);
    if generics.own_params[1..]
        .iter()
        .any(|param| !matches!(param.kind, ty::GenericParamDefKind::Type { .. }))
    {
        return (None, Vec::new());
    }
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    // Fresh variables for the method's whole item, the trait's parameters first and `Self` at index
    // 0, then `Self` pinned to the context.
    let method_args = infcx.fresh_args_for_item(DUMMY_SP, method_did);
    let _ = ocx.eq(
        &ObligationCause::dummy(),
        param_env,
        method_args.type_at(0),
        context,
    );
    // The declared inputs, their late-bound lifetimes instantiated as placeholders first, since
    // relating a type with an escaping bound var panics the generalizer.
    let sig = tcx
        .fn_sig(method_did)
        .instantiate(tcx, method_args)
        .skip_norm_wip();
    let sig = infcx.enter_forall_and_leak_universe(sig);
    // A method's `self` receiver is the first declared input and is not among the call's arguments.
    let inputs = sig.inputs().iter().skip(usize::from(has_self));
    for (arg, declared) in std::iter::zip(args, inputs) {
        if let Some(written) = argument_ty(&infcx, arg) {
            // Best effort: an argument that does not unify (a coerced one) leaves its parameter
            // named.
            let _ = ocx.eq(&ObligationCause::dummy(), param_env, *declared, written);
        }
    }
    let _ = ocx.try_evaluate_obligations();

    let mut unresolved = Vec::new();
    let trait_args = ty::GenericArgs::for_item(tcx, trait_did, |param, _| {
        if param.index == 0 {
            return context.into();
        }
        let resolved =
            infcx.deeply_resolve_ignoring_regions(method_args.type_at(param.index as usize));
        let writable = !resolved.has_infer()
            && !resolved.has_param()
            && !resolved
                .walk()
                .any(|arg| matches!(arg.kind(), ty::GenericArgKind::Lifetime(_)));
        if writable {
            resolved.into()
        } else {
            unresolved.push(param.name.to_string());
            tcx.mk_param_from_def(param)
        }
    });
    let trait_ref = ty::TraitRef::new(tcx, trait_did, trait_args);
    let bound = with_forced_trimmed_paths!(trait_ref.print_only_trait_path().to_string());
    (Some(bound), unresolved)
}

/// The type an argument expression has without typeck: the type it writes ([`expr_written_ty`]),
/// or, for a path to a local binding, the type the binding is declared with (a fn parameter, or a
/// `let` with an annotation).
pub(crate) fn argument_ty<'tcx>(infcx: &InferCtxt<'tcx>, arg: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    if let Some(written) = expr_written_ty(infcx, arg) {
        return Some(written);
    }
    let ExprKind::Path(QPath::Resolved(None, path)) = arg.kind else {
        return None;
    };
    let Res::Local(binding) = path.res else {
        return None;
    };
    let tcx = infcx.tcx;
    if let Node::LetStmt(let_stmt) = tcx.parent_hir_node(binding)
        && let Some(annotation) = let_stmt.ty
    {
        return lower_hir_ty(tcx, annotation);
    }
    // A parameter's late-bound lifetimes (`shape: &Shape`) become fresh region variables, as they
    // would at a call, so the type unifies with a declared input.
    param_declared_input_binder(tcx, binding).map(|declared| {
        infcx.instantiate_binder_with_fresh_vars(
            DUMMY_SP,
            BoundRegionConversionTime::FnCall,
            declared,
        )
    })
}

/// The declared type of the fn parameter `binding` names, with its late-bound lifetimes left bound
/// ([`param_declared_input_binder`]). A `#[cgp_impl]` provider's renamed `self`, declared
/// `&__Context__`, is the case this was written for.
fn param_declared_input<'tcx>(tcx: TyCtxt<'tcx>, binding: HirId) -> Option<Ty<'tcx>> {
    param_declared_input_binder(tcx, binding).map(ty::Binder::skip_binder)
}

/// The declared type of the fn parameter `binding` names, under the signature's binder of
/// late-bound lifetimes, read from the enclosing function's signature rather than from typeck
/// results, which do not exist yet for the body whose type checking is reporting the error. `None`
/// when `binding` is not a parameter.
///
/// An `async fn` body sees its parameters through fresh bindings, since lowering moves each into the
/// future with an untyped `let __context__ = __context__;`, so such a forwarding `let` is followed
/// back to the parameter it copies.
fn param_declared_input_binder<'tcx>(
    tcx: TyCtxt<'tcx>,
    mut binding: HirId,
) -> Option<ty::Binder<'tcx, Ty<'tcx>>> {
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
        .skip_norm_wip();
    let input = *sig.skip_binder().inputs().get(index)?;
    Some(sig.rebind(input))
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

/// Whether `impl_did`'s `where` clause carries a trait bound of trait `trait_did`, so that the
/// impl already declares that trait (via `#[uses]` or a hand-written bound).
fn impl_bounds_by(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId) -> bool {
    tcx.clauses_of(impl_did)
        .clauses
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|bound| bound.skip_binder().trait_ref.def_id == trait_did)
}
