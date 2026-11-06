//! Detecting a `#[cgp_impl]` provider that names the wrong trait.
//!
//! `#[cgp_impl(new RectangleArea)] impl AreaCalculator { … }` turns its header inside out into
//! `impl<__Context__> AreaCalculator<__Context__> for RectangleArea`, inserting the context as the
//! leading generic. Three mistakes name the wrong trait and need different fixes (see
//! [`CgpImplMisuse`]): the header naming the component's
//! *consumer* trait where its *provider* trait belongs, the header naming a trait that is not a CGP
//! component at all, or a higher-order provider's inner-provider `where`-bound (typically from
//! `#[use_provider]`) naming the *consumer* trait: the inner-bound sibling of the first.
//!
//! This module recognizes all three off the compiler, using the consumer- and provider-trait
//! fingerprints. The mistake is confirmed structurally, never from error text, by three conditions
//! that together select the *user's* `#[cgp_impl]` impl and exclude every blanket and forwarding
//! impl the CGP macros generate (which also carry `__Context__`):
//!
//! 1. the impl carries a generic parameter named `__Context__`: the reserved parameter
//!    `#[cgp_impl]` inserts, so this is a macro-generated inside-out impl and not a legitimate
//!    hand-written direct consumer impl (`impl CanGreet for Person`, which has no such parameter);
//! 2. its `Self` is a concrete local struct/enum (the provider struct `RectangleArea`), where the
//!    `#[cgp_component]`-generated consumer and provider *blanket* impls have a bare type-parameter
//!    `Self` (`impl<__Context__> CanCalculateArea for __Context__`); and
//! 3. the header trait reference is a token the user wrote (not from a macro expansion), where the
//!    generated `IsProviderFor` / `DelegateComponent` forwarding impls carry a macro-synthesized
//!    trait reference.
//!
//! The header trait is then classified: a CGP *consumer* trait ([`consumer_provider_trait`] yields
//! its paired provider, a `ConsumerTrait` misuse) or neither a consumer nor a provider trait (a
//! `NonCgpTrait` misuse). A header that *is* the provider trait is the correct form; even then,
//! each of the impl's inner-provider `where`-bounds is scanned for a consumer trait (a
//! `ConsumerProviderBound` misuse). An inner-provider bound is one on a type parameter other than
//! `__Context__` whose trait takes `__Context__` as its first argument after any lifetimes, the
//! shape `#[use_provider]` writes; a `#[uses]` bound on the context itself is never one, so its own
//! `E0107` (an arity slip) keeps rustc's wording. Each candidate carries its own trait-reference
//! span, so the `E0107` that lands on one selects which is reshaped.
//!
//! No condition forces a query on the malformed impl itself: the header trait's `DefId` and `Self`'s
//! kind are read from HIR, and the fingerprint queries touch only the well-formed consumer/provider
//! blanket impls `#[cgp_component]` generates.
//!
//! **Boundary.** Recognition is deliberately specific to `#[cgp_impl]`, whose macro-inserted
//! `__Context__` parameter is the marker used above. The lower-level `#[cgp_provider]` /
//! `#[cgp_new_provider]` forms, which spell the whole inside-out impl by hand with a user-named
//! context parameter, are *not* covered, and cannot be safely: without the reserved `__Context__`
//! marker, `impl<Ctx> SomeConsumer<Ctx> for ConcreteType` is indistinguishable from a legitimate
//! hand-written direct impl of a generic consumer trait on a context, so flagging it would risk a
//! false positive on valid code. `#[cgp_impl]` is the idiomatic provider form, so this covers the
//! case a programmer is overwhelmingly likely to hit.

use cargo_cgp_error_processing::CgpImplMisuse;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{
    GenericArg, GenericBound, ItemKind, PathSegment, QPath, Ty, TyKind, WherePredicate,
    WherePredicateKind,
};
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::LocalDefId;
use rustc_span::{Span, Symbol};

use crate::config::CGP_IMPL_CONTEXT_PARAM;
use crate::resolve::query::{
    consumer_provider_trait, context_param_span, is_provider_trait, local_trait_impls,
};

/// A `#[cgp_impl]` provider impl whose header names the wrong trait. Carries the rustc-free wording
/// model plus the spans the emitter needs to reshape the precise error and suppress the rest of the
/// cascade the one mistake produces.
pub struct DetectedCgpImplMisuse {
    /// The consumer/provider (or non-CGP trait) names, worded into the coded header and help.
    pub misuse: CgpImplMisuse,
    /// The impl's self type (the provider struct `RectangleArea`), used to tie the downstream
    /// `NotAProvider` check re-report back to this mistake.
    pub self_ty: String,
    /// The span of the header trait reference, matched against the `E0107` whose caret lands there so
    /// that diagnostic is the one reshaped.
    pub trait_ref_span: Span,
    /// The full source span of the impl block, matched against a sibling macro-lowering error whose
    /// caret lands *inside the impl body* (`E0186` on the method) so it is suppressed as a
    /// consequence of the one mistake.
    pub impl_span: Span,
    /// The span of the macro-inserted `__Context__` parameter: the `#[cgp_impl(…)]` attribute
    /// call-site every synthesized token shares. It is where the siblings landing *outside* the impl
    /// body sit (`E0425` on the missing `…Component` marker, `E0207` on the unconstrained
    /// `__Context__`), so it is the second span sibling errors are matched against.
    pub macro_span: Span,
}

/// Recover every `#[cgp_impl]` header-trait mistake in the crate, reading each structurally off the
/// compiler with the fingerprints described in the module docs.
pub fn detect_cgp_impl_misuses(tcx: TyCtxt<'_>) -> Vec<DetectedCgpImplMisuse> {
    local_trait_impls(tcx)
        .flat_map(|local| impl_misuses(tcx, local))
        .collect()
}

/// The mistakes of one impl: its header's, and each inner-provider bound's.
fn impl_misuses(tcx: TyCtxt<'_>, local: LocalDefId) -> Vec<DetectedCgpImplMisuse> {
    let mut found = Vec::new();
    let item = tcx.hir_expect_item(local);
    let ItemKind::Impl(imp) = item.kind else {
        return Vec::new();
    };
    // (1) The `#[cgp_impl]` fingerprint: a leading context type parameter named `__Context__`,
    // whose own span is the `#[cgp_impl(…)]` attribute call-site the sibling errors land on.
    let Some(macro_span) = context_param_span(tcx, local) else {
        return Vec::new();
    };
    // (2) `Self` is a concrete local struct/enum (the provider struct), not the bare type
    // parameter the generated consumer/provider blanket impls use.
    let TyKind::Path(QPath::Resolved(_, self_path)) = imp.self_ty.kind else {
        return Vec::new();
    };
    let Some(self_did) = self_path.res.opt_def_id() else {
        return Vec::new();
    };
    if !matches!(
        tcx.def_kind(self_did),
        DefKind::Struct | DefKind::Enum | DefKind::Union
    ) {
        return Vec::new();
    }
    let Some(header) = imp.of_trait else {
        return Vec::new();
    };
    let Some(header_segment) = header.trait_ref.path.segments.last() else {
        return Vec::new();
    };
    let trait_ref_span = header_segment.ident.span;
    // (3) The header trait reference is a token the user wrote, not the macro-synthesized
    // reference every generated forwarding impl carries.
    if trait_ref_span.from_expansion() {
        return Vec::new();
    }
    let Some(header_did) = header.trait_ref.path.res.opt_def_id() else {
        return Vec::new();
    };
    let self_ty = tcx.item_name(self_did).to_string();
    // Classify the header by the trait's fingerprint. A consumer trait pairs with a provider
    // trait to suggest (`CGP-E032`); a non-component is `CGP-E033`; the provider trait is the *correct*
    // header, leaving only a possible inner-bound mistake below.
    let header_misuse = if let Some(provider_did) = consumer_provider_trait(tcx, header_did) {
        Some(CgpImplMisuse::ConsumerTrait {
            consumer: tcx.item_name(header_did).to_string(),
            provider: tcx.item_name(provider_did).to_string(),
            arguments: written_arguments(tcx, header_segment),
        })
    } else if is_provider_trait(tcx, header_did) {
        None
    } else {
        Some(CgpImplMisuse::NonCgpTrait {
            trait_name: tcx.item_name(header_did).to_string(),
        })
    };
    if let Some(misuse) = header_misuse {
        found.push(DetectedCgpImplMisuse {
            misuse,
            self_ty: self_ty.clone(),
            trait_ref_span,
            impl_span: item.span,
            macro_span,
        });
    }
    found.extend(inner_bound_misuses(
        tcx,
        imp.generics.predicates,
        &self_ty,
        item.span,
        macro_span,
    ));
    found
}

/// A higher-order provider's inner-provider bound (typically written through
/// `#[use_provider]`, which supplies the leading context argument) that names the
/// component's *consumer* trait: the same consumer/provider confusion as the header case, in
/// the `where` clause (`CGP-E034`). Every such bound is a candidate; the `E0107` that lands on its
/// trait name selects which one is reshaped. A bound on the context, which `#[uses]` writes, is
/// skipped (see [`is_inner_provider_bound`]).
fn inner_bound_misuses(
    tcx: TyCtxt<'_>,
    predicates: &[WherePredicate<'_>],
    self_ty: &str,
    impl_span: Span,
    macro_span: Span,
) -> Vec<DetectedCgpImplMisuse> {
    let mut found = Vec::new();
    for predicate in predicates {
        let WherePredicateKind::BoundPredicate(bound) = predicate.kind else {
            continue;
        };
        for generic_bound in bound.bounds {
            let GenericBound::Trait(poly) = generic_bound else {
                continue;
            };
            let bound_path = poly.trait_ref.path;
            let Some(bound_did) = bound_path.res.opt_def_id() else {
                continue;
            };
            let Some(provider_did) = consumer_provider_trait(tcx, bound_did) else {
                continue;
            };
            let Some(segment) = bound_path.segments.last() else {
                continue;
            };
            if !is_inner_provider_bound(bound.bounded_ty, segment) {
                continue;
            }
            found.push(DetectedCgpImplMisuse {
                misuse: CgpImplMisuse::ConsumerProviderBound {
                    consumer: tcx.item_name(bound_did).to_string(),
                    provider: tcx.item_name(provider_did).to_string(),
                    arguments: written_arguments(tcx, segment),
                },
                self_ty: self_ty.to_owned(),
                trait_ref_span: segment.ident.span,
                impl_span,
                macro_span,
            });
        }
    }
    found
}

/// Whether a `where` bound has the shape `#[use_provider]` writes, `Inner: Trait<__Context__, …>`:
/// its bounded type is a type parameter other than `__Context__`, and its trait's first argument
/// after any lifetimes is `__Context__`, where the attribute inserts it
/// (`Inner: CanRead<'a, __Context__, Source>`). A consumer trait in that shape receives the context
/// it cannot take, the mistake `CGP-E034` names. A bound on `__Context__` itself, as `#[uses]`
/// writes, is a dependency of the context, so a wrong argument count there is an ordinary arity
/// slip; and a bound on an inner provider whose first such argument is another type is the
/// program's own, not the attribute's.
fn is_inner_provider_bound(bounded: &Ty<'_>, segment: &PathSegment<'_>) -> bool {
    let is_context = |name: Symbol| name.as_str() == CGP_IMPL_CONTEXT_PARAM;
    let Some(bounded) = type_param_name(bounded) else {
        return false;
    };
    let first_after_lifetimes = segment.args.and_then(|args| {
        args.args
            .iter()
            .find(|arg| !matches!(arg, GenericArg::Lifetime(_)))
    });
    let Some(GenericArg::Type(first)) = first_after_lifetimes else {
        return false;
    };
    !is_context(bounded) && type_param_name(first.as_unambig_ty()).is_some_and(is_context)
}

/// The generic arguments the program wrote on a trait reference that `#[cgp_impl]` or
/// `#[use_provider]` gave the context, each read from its source: every argument and constraint but
/// the inserted `__Context__`, which follows any lifetimes (`CanRead<'a, __Context__, Text>`).
/// Empty when an argument's source cannot be read, so the help names the bare trait.
fn written_arguments(tcx: TyCtxt<'_>, segment: &PathSegment<'_>) -> Vec<String> {
    let Some(args) = segment.args else {
        return Vec::new();
    };
    let source_map = tcx.sess.source_map();
    let is_context = |arg: &&GenericArg<'_>| match arg {
        GenericArg::Type(ty) => type_param_name(ty.as_unambig_ty())
            .is_some_and(|name| name.as_str() == CGP_IMPL_CONTEXT_PARAM),
        _ => false,
    };
    let arguments = args
        .args
        .iter()
        .filter(|arg| !is_context(arg))
        .map(GenericArg::span);
    let constraints = args.constraints.iter().map(|constraint| constraint.span);
    arguments
        .chain(constraints)
        .map(|span| source_map.span_to_snippet(span).ok())
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

/// The name of the generic type parameter a HIR type is, or `None` for any other type.
fn type_param_name(ty: &Ty<'_>) -> Option<Symbol> {
    let TyKind::Path(QPath::Resolved(None, path)) = ty.kind else {
        return None;
    };
    let Res::Def(DefKind::TyParam, _) = path.res else {
        return None;
    };
    match path.segments {
        [segment] => Some(segment.ident.name),
        _ => None,
    }
}
