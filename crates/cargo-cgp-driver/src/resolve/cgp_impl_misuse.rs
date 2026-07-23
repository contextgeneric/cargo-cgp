//! Detecting a `#[cgp_impl]` provider that names the wrong trait.
//!
//! `#[cgp_impl(new RectangleArea)] impl AreaCalculator { … }` turns its header inside out into
//! `impl<__Context__> AreaCalculator<__Context__> for RectangleArea`, inserting the context as the
//! leading generic. Three mistakes name the wrong trait and need different fixes (see
//! [`CgpImplMisuse`](cargo_cgp_error_processing::CgpImplMisuse)): the header naming the component's
//! *consumer* trait where its *provider* trait belongs, the header naming a trait that is not a CGP
//! component at all, or a higher-order provider's inner-provider `where`-bound (typically from
//! `#[use_provider]`) naming the *consumer* trait — the inner-bound sibling of the first.
//!
//! This module recognizes all three off the compiler, using the consumer- and provider-trait
//! fingerprints. The mistake is confirmed structurally, never from error text, by three conditions
//! that together select the *user's* `#[cgp_impl]` impl and exclude every blanket and forwarding
//! impl the CGP macros generate (which also carry `__Context__`):
//!
//! 1. the impl carries a generic parameter named `__Context__` — the reserved parameter
//!    `#[cgp_impl]` inserts, so this is a macro-generated inside-out impl and not a legitimate
//!    hand-written direct consumer impl (`impl CanGreet for Person`, which has no such parameter);
//! 2. its `Self` is a concrete local struct/enum — the provider struct (`RectangleArea`) — where the
//!    `#[cgp_component]`-generated consumer and provider *blanket* impls have a bare type-parameter
//!    `Self` (`impl<__Context__> CanCalculateArea for __Context__`); and
//! 3. the header trait reference is a token the user wrote (not from a macro expansion), where the
//!    generated `IsProviderFor` / `DelegateComponent` forwarding impls carry a macro-synthesized
//!    trait reference.
//!
//! The header trait is then classified: a CGP *consumer* trait ([`consumer_provider_trait`] yields
//! its paired provider — a `ConsumerTrait` misuse) or neither a consumer nor a provider trait (a
//! `NonCgpTrait` misuse). A header that *is* the provider trait is the correct form; even then, each
//! of the impl's inner-provider `where`-bounds is scanned for a consumer trait (a
//! `ConsumerProviderBound` misuse). Each candidate carries its own trait-reference span, so the
//! `E0107` that lands on one selects which is reshaped.
//!
//! No condition forces a query on the malformed impl itself: the header trait's `DefId` and `Self`'s
//! kind are read from HIR, and the fingerprint queries touch only the well-formed consumer/provider
//! blanket impls `#[cgp_component]` generates.
//!
//! **Boundary.** Recognition is deliberately specific to `#[cgp_impl]`, whose macro-inserted
//! `__Context__` parameter is the marker used above. The lower-level `#[cgp_provider]` /
//! `#[cgp_new_provider]` forms — which spell the whole inside-out impl by hand with a user-named
//! context parameter — are *not* covered, and cannot be safely: without the reserved `__Context__`
//! marker, `impl<Ctx> SomeConsumer<Ctx> for ConcreteType` is indistinguishable from a legitimate
//! hand-written direct impl of a generic consumer trait on a context, so flagging it would risk a
//! false positive on valid code. `#[cgp_impl]` is the idiomatic provider form, so this covers the
//! case a programmer is overwhelmingly likely to hit.

use cargo_cgp_error_processing::CgpImplMisuse;
use rustc_hir::def::DefKind;
use rustc_hir::{
    GenericArg, GenericBound, GenericParamKind, ItemKind, ParamName, QPath, TyKind,
    WherePredicateKind,
};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::resolve::cgp_item::{consumer_provider_trait, is_provider_trait};

/// The reserved generic parameter `#[cgp_impl]` inserts as the leading context of the provider impl
/// it generates. Its presence is the fingerprint of a macro-generated inside-out impl, telling it
/// apart from a legitimate hand-written direct consumer-trait impl (`impl CanGreet for Person`).
const CGP_IMPL_CONTEXT_PARAM: &str = "__Context__";

/// A `#[cgp_impl]` provider impl whose header names the wrong trait. Carries the rustc-free wording
/// model plus the spans the emitter needs to reshape the precise error and suppress the rest of the
/// cascade the one mistake produces.
pub struct DetectedCgpImplMisuse {
    /// The consumer/provider (or non-CGP trait) names, worded into the coded header and help.
    pub misuse: CgpImplMisuse,
    /// The impl's self type — the provider struct (`RectangleArea`) — used to tie the downstream
    /// `NotAProvider` check re-report back to this mistake.
    pub self_ty: String,
    /// The span of the header trait reference, matched against the `E0107` whose caret lands there so
    /// that diagnostic is the one reshaped.
    pub trait_ref_span: Span,
    /// The full source span of the impl block, matched against a sibling macro-lowering error whose
    /// caret lands *inside the impl body* (`E0186` on the method) so it is suppressed as a
    /// consequence of the one mistake.
    pub impl_span: Span,
    /// The span of the macro-inserted `__Context__` parameter — the `#[cgp_impl(…)]` attribute
    /// call-site every synthesized token shares. It is where the siblings landing *outside* the impl
    /// body sit (`E0425` on the missing `…Component` marker, `E0207` on the unconstrained
    /// `__Context__`), so it is the second span sibling errors are matched against.
    pub macro_span: Span,
}

/// Recover every `#[cgp_impl]` header-trait mistake in the crate, reading each structurally off the
/// compiler with the fingerprints described in the module docs.
pub fn detect_cgp_impl_misuses(tcx: TyCtxt<'_>) -> Vec<DetectedCgpImplMisuse> {
    let mut found = Vec::new();
    for local in tcx.hir_crate_items(()).definitions() {
        if !matches!(tcx.def_kind(local), DefKind::Impl { of_trait: true }) {
            continue;
        }
        let item = tcx.hir_expect_item(local);
        let ItemKind::Impl(imp) = item.kind else {
            continue;
        };
        // (1) The `#[cgp_impl]` fingerprint: a leading context type parameter named `__Context__`,
        // whose own span is the `#[cgp_impl(…)]` attribute call-site the sibling errors land on.
        let Some(macro_span) = imp.generics.params.iter().find_map(|param| {
            (matches!(param.kind, GenericParamKind::Type { .. })
                && matches!(param.name, ParamName::Plain(ident) if ident.name.as_str() == CGP_IMPL_CONTEXT_PARAM))
            .then_some(param.span)
        }) else {
            continue;
        };
        // (2) `Self` is a concrete local struct/enum — the provider struct — not the bare type
        // parameter the generated consumer/provider blanket impls use.
        let TyKind::Path(QPath::Resolved(_, self_path)) = imp.self_ty.kind else {
            continue;
        };
        let Some(self_did) = self_path.res.opt_def_id() else {
            continue;
        };
        if !matches!(
            tcx.def_kind(self_did),
            DefKind::Struct | DefKind::Enum | DefKind::Union
        ) {
            continue;
        }
        let Some(header) = imp.of_trait else {
            continue;
        };
        let trait_ref_span = header
            .trait_ref
            .path
            .segments
            .last()
            .map_or(header.trait_ref.path.span, |segment| segment.ident.span);
        // (3) The header trait reference is a token the user wrote — not the macro-synthesized
        // reference every generated forwarding impl carries.
        if trait_ref_span.from_expansion() {
            continue;
        }
        let Some(header_did) = header.trait_ref.path.res.opt_def_id() else {
            continue;
        };
        let self_ty = tcx.item_name(self_did).to_string();
        // Classify the header by the trait's fingerprint. A consumer trait pairs with a provider
        // trait to suggest (E013); a non-component is E014; the provider trait is the *correct*
        // header, leaving only a possible inner-bound mistake below.
        let header_misuse = if let Some(provider_did) = consumer_provider_trait(tcx, header_did) {
            Some(CgpImplMisuse::ConsumerTrait {
                consumer: tcx.item_name(header_did).to_string(),
                provider: tcx.item_name(provider_did).to_string(),
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
        // A higher-order provider's inner-provider bound — typically written through
        // `#[use_provider]`, which supplies the leading context argument — that names the
        // component's *consumer* trait: the same consumer/provider confusion as the header case, in
        // the `where` clause (E015). Every such bound is a candidate; the `E0107` that lands on its
        // trait name selects which one is reshaped.
        for predicate in imp.generics.predicates {
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
                // Only a consumer trait *given generic arguments* is the mistake — the inserted
                // `<Self>` the consumer trait cannot take. A bare `Self: SomeConsumer` self-bound is
                // legitimate and produces no `E0107`.
                let has_type_arg = segment.args.is_some_and(|args| {
                    args.args
                        .iter()
                        .any(|arg| matches!(arg, GenericArg::Type(_)))
                });
                if !has_type_arg {
                    continue;
                }
                found.push(DetectedCgpImplMisuse {
                    misuse: CgpImplMisuse::ConsumerProviderBound {
                        consumer: tcx.item_name(bound_did).to_string(),
                        provider: tcx.item_name(provider_did).to_string(),
                    },
                    self_ty: self_ty.clone(),
                    trait_ref_span: segment.ident.span,
                    impl_span: item.span,
                    macro_span,
                });
            }
        }
    }
    found
}
