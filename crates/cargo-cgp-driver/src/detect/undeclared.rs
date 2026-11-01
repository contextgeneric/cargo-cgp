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
//! recognized by its receiver: a parameter or a `let` declared as that context, or the context named
//! as the qualifying type. A call on a value declared with another type (`value: &Value`) is left
//! alone, since its fix is a bound on that type, which `#[uses]` cannot express.
//!
//! This module recognizes that shape off the compiler: the failing call sits inside a generated
//! impl over a bare type-parameter context and is made on that context, the call names a method of
//! a CGP trait, and that trait is *not* among the impl's `where` bounds. It fills the rustc-free
//! [`UndeclaredTrait`] the emitter words into a `[CGP-E030]` header and `#[uses(…)]` help, with the
//! trait written with the arguments the call fixes.

use cargo_cgp_error_processing::UndeclaredTrait;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, HirId, QPath};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_middle::ty::print::{PrintTraitRefExt as _, with_forced_trimmed_paths};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _, TypingMode};
use rustc_span::def_id::DefId;
use rustc_span::{DUMMY_SP, Span, Symbol};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

use crate::resolve::query::{
    argument_ty, binding_let_annotation, enclosing_trait_impl, is_blanket_trait,
    is_cgp_definition_impl, is_consumer_trait, is_provider_trait, local_binding, local_body_exprs,
    param_declared_input, peel_hir_refs, trait_items_named, traits_with_method, unify_call_inputs,
};

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
    // Only `#[cgp_fn]` and `#[cgp_impl]` take `#[uses]`. A hand-written impl over a type parameter,
    // with or without `#[cgp_provider]`, fixes the call with an ordinary bound.
    if !is_cgp_definition_impl(tcx, impl_did) {
        return None;
    }
    let (context, self_is_context) = generated_context(trait_ref)?;
    let blanket_impl = self_is_context.then_some(impl_did);
    if !called_on_context(tcx, &call.receiver, context, blanket_impl) {
        return None;
    }

    for (trait_did, method_did) in candidate_traits(tcx, &call, spans) {
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

/// The generic context a generated impl is over, and whether it is the impl's `Self`: a `#[cgp_fn]`
/// blanket impl's `Self`, or a
/// `#[cgp_impl]` provider impl's first trait argument (`Greeter<__Context__> for GreetHello`).
/// A hand-written `impl … for ConcreteContext` has neither and is a different
/// (concrete-context) failure.
fn generated_context(trait_ref: ty::TraitRef<'_>) -> Option<(Ty<'_>, bool)> {
    if matches!(trait_ref.self_ty().kind(), ty::Param(_)) {
        return Some((trait_ref.self_ty(), true));
    }
    // `args` begins with `Self`, so the provider trait's own first argument is the second.
    let first = trait_ref.args.types().nth(1)?;
    matches!(first.kind(), ty::Param(_)).then_some((first, false))
}

/// Whether the call is on the generated `context`; `blanket_impl` is the impl when its `Self` is the
/// context. A receiver declared with another type (a `value: &Value` parameter, or a `let` annotated
/// that way) needs a bound on `Value`, which `#[uses]` cannot give, so it is not this failure. A receiver whose type cannot be read off a signature (a method call's
/// result) is accepted only in a `#[cgp_fn]` impl, whose `Self` is the context.
fn called_on_context<'tcx>(
    tcx: TyCtxt<'tcx>,
    receiver: &CallReceiver<'tcx>,
    context: Ty<'tcx>,
    blanket_impl: Option<DefId>,
) -> bool {
    match *receiver {
        CallReceiver::Value(binding) => {
            if let Some(annotation) =
                binding.and_then(|binding| binding_let_annotation(tcx, binding))
            {
                return names_context(tcx, peel_hir_refs(annotation), context, blanket_impl);
            }
            match binding.and_then(|binding| param_declared_input(tcx, binding)) {
                Some(declared) => declared.peel_refs() == context,
                None => blanket_impl.is_some(),
            }
        }
        CallReceiver::Type(qself) => names_context(tcx, qself, context, blanket_impl),
    }
}

/// The CGP trait(s) declaring `method`. Several unrelated traits can share a method name
/// across modules (a `#[cgp_fn]` trait and a `#[cgp_component]` consumer both named
/// `fetch_storage_object`), so disambiguate to the one the diagnostic actually points at: the
/// "trait bound not satisfied" note spans the *failing* trait's own definition. Only if none is
/// referenced (a single unambiguous candidate) fall back to the full list.
fn candidate_traits(
    tcx: TyCtxt<'_>,
    call: &FailingCall<'_>,
    spans: &[Span],
) -> Vec<(DefId, DefId)> {
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
    if referenced.is_empty() {
        candidates
    } else {
        referenced
    }
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
    local_body_exprs(tcx)
        .filter_map(|expr| match expr.kind {
            ExprKind::MethodCall(segment, receiver, args, _)
                if segment.ident.span.overlaps(primary_span) =>
            {
                Some(FailingCall {
                    method: segment.ident.name,
                    hir_id: expr.hir_id,
                    receiver: CallReceiver::Value(local_binding(receiver)),
                    args,
                })
            }
            ExprKind::Call(callee, args) => {
                if let ExprKind::Path(QPath::TypeRelative(qself, segment)) = callee.kind
                    && segment.ident.span.overlaps(primary_span)
                {
                    Some(FailingCall {
                        method: segment.ident.name,
                        hir_id: expr.hir_id,
                        receiver: CallReceiver::Type(qself),
                        args,
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .last()
}

/// Whether a written type, the qualifying type of an associated-function call or a `let`'s annotation
/// with its references peeled, names the generated context: a `#[cgp_impl]` body's `Self`, which the macro writes as the `__Context__` parameter, or
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
    for (trait_did, method_did) in trait_items_named(tcx, method, |item| {
        matches!(item.kind, ty::AssocKind::Fn { .. })
    }) {
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
    // A method's `self` receiver is the first declared input and is not among the call's arguments.
    // An argument that does not unify (a coerced one) leaves its parameter named.
    let skip = usize::from(has_self);
    unify_call_inputs(&ocx, method_did, method_args, args, skip, |arg| {
        argument_ty(&infcx, arg)
    });
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

/// Whether `impl_did`'s `where` clause carries a trait bound of trait `trait_did`, so that the
/// impl already declares that trait (via `#[uses]` or a hand-written bound).
fn impl_bounds_by(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId) -> bool {
    tcx.clauses_of(impl_did)
        .clauses
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|bound| bound.skip_binder().trait_ref.def_id == trait_did)
}
