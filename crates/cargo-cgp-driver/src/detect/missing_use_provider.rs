//! Detecting a higher-order provider that calls an inner provider it never imported.
//!
//! A higher-order provider names an inner provider as a generic parameter and invokes it as an
//! associated function on the context: `InnerCalculator::area(self)`. For that call to resolve, the
//! inner parameter must carry the provider-trait bound `InnerCalculator: AreaCalculator<Self>`,
//! declared idiomatically with `#[use_provider(InnerCalculator: AreaCalculator)]`. When the import is
//! forgotten, the parameter is unbounded and rustc reports a vague `E0599` ("no associated function
//! `area` found for type parameter `InnerCalculator` … due to unsatisfied trait bounds"), whose
//! suggestion leaks the generated `__Context__` and offers the *consumer* trait as a bound, the wrong
//! fix for a higher-order provider.
//!
//! This module recognizes that shape off the compiler: the failing call is an associated-function
//! call `Param::method(…)` whose `Param` is a generic parameter of an enclosing provider-trait impl,
//! the called method belongs to a CGP *provider* trait, and `Param` is not bounded by that provider
//! trait. It fills the rustc-free [`MissingUseProvider`] the emitter words into a `[CGP-E031]` header
//! and `#[use_provider(…)]` help, with a generic component's parameters recovered from the call.

use cargo_cgp_error_processing::MissingUseProvider;
use rustc_hir::def::DefKind;
use rustc_hir::{Expr, QPath, TyKind};
use rustc_middle::ty::print::with_forced_trimmed_paths;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::DefId;

use crate::resolve::query::{
    CallAt, call_fixed_arguments, calls_at, enclosing_trait_impl, is_cgp_definition_impl,
    is_provider_trait, trait_items_named,
};

/// Recognize a missing-`#[use_provider]` failure and recover the inner provider and the provider
/// trait to import it as, or `None` when the diagnostic is not one. Everything is keyed on
/// `primary_span`, the failing associated-function call.
pub fn detect_missing_use_provider(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<MissingUseProvider> {
    let FailingAssocFnCall {
        inner,
        inner_did,
        method,
        hir_id: call_hir_id,
        args,
    } = failing_assoc_fn_call_at(tcx, primary_span)?;

    // `inner` must be a generic type parameter (a higher-order provider's inner-provider slot), not
    // a concrete type (whose associated-function failure is a different mistake).
    if !matches!(tcx.def_kind(inner_did), DefKind::TyParam) {
        return None;
    }

    // The enclosing trait impl, and its own provider trait (a higher-order provider implements one).
    let owner = tcx.hir_enclosing_body_owner(call_hir_id);
    let impl_did = enclosing_trait_impl(tcx, owner.to_def_id())?;
    // `#[use_provider]` goes on a definition a CGP macro lowers. A type parameter of a hand-written
    // impl, with or without `#[cgp_provider]`, is an ordinary one, whose fix is an ordinary bound.
    if !is_cgp_definition_impl(tcx, impl_did) {
        return None;
    }
    let impl_trait_ref = tcx.impl_trait_ref(impl_did).skip_binder();
    let impl_trait_did = impl_trait_ref.def_id;

    // The provider impl's own context parameter is what `Self` lowers to in a `#[cgp_impl]` body, so
    // `Self::raise_error(…)` names it; it is the context, never an inner provider, and its missing
    // bound is a `#[uses]` dependency rather than a `#[use_provider]` import.
    if is_provider_trait(tcx, impl_trait_did)
        && let Some(context) = impl_trait_ref.args.types().nth(1)
        && matches!(context.kind(), ty::Param(p) if p.name.as_str() == inner)
    {
        return None;
    }

    let provider_did = called_provider_trait(tcx, impl_trait_did, method)?;

    // If `inner` is already bounded by that provider trait, the call would resolve, so this `E0599`
    // is some other failure, not a forgotten import.
    if impl_bounds_param_by(tcx, impl_did, provider_did, &inner) {
        return None;
    }

    let (provider_trait, unresolved) =
        use_provider_bound(tcx, impl_did, provider_did, inner_did, method, args);
    Some(MissingUseProvider {
        inner,
        provider_trait,
        unresolved,
    })
}

/// The provider trait the call's method belongs to: the enclosing impl's own trait when it
/// declares the method (a higher-order provider wrapping the same component, the common case),
/// else the sole provider trait that declares it.
fn called_provider_trait(
    tcx: TyCtxt<'_>,
    impl_trait_did: DefId,
    method: rustc_span::Symbol,
) -> Option<DefId> {
    if is_provider_trait(tcx, impl_trait_did) && trait_has_method(tcx, impl_trait_did, method) {
        return Some(impl_trait_did);
    }
    let mut candidates = trait_items_named(tcx, method, |item| {
        matches!(item.kind, ty::AssocKind::Fn { .. })
    })
    .map(|(trait_did, _)| trait_did)
    .filter(|&trait_did| is_provider_trait(tcx, trait_did));
    let only = candidates.next()?;
    candidates.next().is_none().then_some(only)
}

/// The bound `#[use_provider]` declares for the inner provider of the impl `impl_did`, and the trait
/// parameters it leaves named.
fn use_provider_bound<'tcx>(
    tcx: TyCtxt<'tcx>,
    impl_did: DefId,
    provider_did: DefId,
    inner_did: DefId,
    method: rustc_span::Symbol,
    args: &[Expr<'tcx>],
) -> (Option<String>, Vec<String>) {
    let impl_trait_ref = tcx.impl_trait_ref(impl_did).skip_binder();
    // The context the impl is over, which the inner provider is called with: the provider trait's
    // first argument after its `Self`.
    let context = is_provider_trait(tcx, impl_trait_ref.def_id)
        .then(|| impl_trait_ref.args.types().nth(1))
        .flatten();
    let method_did = tcx
        .associated_items(provider_did)
        .filter_by_name_unhygienic(method)
        .find(|item| matches!(item.kind, ty::AssocKind::Fn { .. }))
        .map(|item| item.def_id);
    // The inner provider as a type: the impl's own generic parameter the call qualifies by.
    let impl_generics = tcx.generics_of(impl_did);
    let inner_ty = impl_generics
        .param_def_id_to_index(tcx, inner_did)
        .map(|index| tcx.mk_param_from_def(impl_generics.param_at(index as usize, tcx)))
        .and_then(|arg| arg.as_type());
    match (context, method_did, inner_ty) {
        (Some(context), Some(method_did), Some(inner_ty)) => {
            written_provider_bound(tcx, provider_did, method_did, inner_ty, context, args)
        }
        _ => (bare_provider_bound(tcx, provider_did), Vec::new()),
    }
}

/// The provider trait as `#[use_provider]` writes it, `AreaCalculator<Shape>`: the context argument
/// left out, since the attribute supplies it, and each further argument the one the call fixes
/// ([`call_fixed_arguments`](fn@call_fixed_arguments), with `Self` pinned to the inner provider and
/// the context to the impl's). An argument may be one of the impl's own generic parameters, which
/// the attribute on the same impl can name. A parameter that stays unknown, or comes out holding a
/// lifetime, is written as the trait's own parameter name for the reader to replace. `None` when
/// the trait takes a lifetime or const parameter, which the help cannot write.
fn written_provider_bound<'tcx>(
    tcx: TyCtxt<'tcx>,
    provider_did: DefId,
    method_did: DefId,
    inner: Ty<'tcx>,
    context: Ty<'tcx>,
    args: &[Expr<'tcx>],
) -> (Option<String>, Vec<String>) {
    let name = tcx.item_name(provider_did).to_string();
    let generics = tcx.generics_of(provider_did);
    if generics.own_params.len() <= 2
        || generics.own_params[1..]
            .iter()
            .any(|param| !matches!(param.kind, ty::GenericParamDefKind::Type { .. }))
    {
        return (bare_provider_bound(tcx, provider_did), Vec::new());
    }
    let fixed = call_fixed_arguments(
        tcx,
        method_did,
        &[inner, context],
        args,
        0,
        &generics.own_params[2..],
        true,
    );
    let mut unresolved = Vec::new();
    let rendered: Vec<String> = std::iter::zip(&generics.own_params[2..], fixed)
        .map(|(param, resolved)| match resolved {
            Some(resolved) => with_forced_trimmed_paths!(resolved.to_string()),
            None => {
                unresolved.push(param.name.to_string());
                param.name.to_string()
            }
        })
        .collect();
    (Some(format!("{name}<{}>", rendered.join(", "))), unresolved)
}

/// The provider trait by its name alone, which `#[use_provider]` completes with the context: valid
/// only for a trait whose one parameter besides `Self` is the context.
fn bare_provider_bound(tcx: TyCtxt<'_>, provider_did: DefId) -> Option<String> {
    (tcx.generics_of(provider_did).own_params.len() <= 2)
        .then(|| tcx.item_name(provider_did).to_string())
}

/// The associated-function call `Param::method(…)` the primary span points at.
struct FailingAssocFnCall<'tcx> {
    /// The inner provider's name, as the call writes it.
    inner: String,
    /// The `DefId` the qualifying path resolves to.
    inner_did: DefId,
    /// The called function's name.
    method: rustc_span::Symbol,
    /// The call expression.
    hir_id: rustc_hir::HirId,
    /// The call's arguments.
    args: &'tcx [Expr<'tcx>],
}

/// The associated-function call `Param::method(…)` the primary span points at, matched precisely on
/// the method segment's own span.
fn failing_assoc_fn_call_at(tcx: TyCtxt<'_>, primary_span: Span) -> Option<FailingAssocFnCall<'_>> {
    calls_at(tcx, primary_span)
        .filter_map(|call| {
            if let CallAt::TypeRelative {
                hir_id,
                qself,
                segment,
                args,
            } = call
                && let TyKind::Path(QPath::Resolved(_, path)) = qself.kind
                && let Some(param_did) = path.res.opt_def_id()
            {
                let name = path
                    .segments
                    .last()
                    .map(|seg| seg.ident.name.to_string())
                    .unwrap_or_default();
                Some(FailingAssocFnCall {
                    inner: name,
                    inner_did: param_did,
                    method: segment.ident.name,
                    hir_id,
                    args,
                })
            } else {
                None
            }
        })
        .last()
}

/// Whether `trait_did` declares an associated function named `method`.
fn trait_has_method(tcx: TyCtxt<'_>, trait_did: DefId, method: rustc_span::Symbol) -> bool {
    tcx.associated_items(trait_did)
        .filter_by_name_unhygienic(method)
        .any(|item| matches!(item.kind, ty::AssocKind::Fn { .. }))
}

/// Whether `impl_did`'s `where` clause bounds a type parameter named `param` by trait `trait_did`:
/// i.e. the inner provider is already imported (via `#[use_provider]` or a hand-written bound).
fn impl_bounds_param_by(tcx: TyCtxt<'_>, impl_did: DefId, trait_did: DefId, param: &str) -> bool {
    tcx.clauses_of(impl_did)
        .clauses
        .iter()
        .filter_map(|(clause, _)| clause.as_trait_clause())
        .any(|bound| {
            let bound = bound.skip_binder();
            bound.trait_ref.def_id == trait_did
                && matches!(bound.self_ty().kind(), ty::Param(p) if p.name.as_str() == param)
        })
}
