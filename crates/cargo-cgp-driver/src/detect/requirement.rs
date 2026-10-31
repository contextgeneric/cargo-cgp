//! Detecting a component requirement a `#[cgp_impl]` provider does not declare.
//!
//! A component's `#[extend(HasName)]` or `#[use_type(HasErrorType.Error)]` lowers to a `where`
//! clause on the provider trait (`trait Greeter<__Context__> where __Context__: HasName`), so every
//! provider must declare the bound. `#[cgp_impl]` writes a provider's declarations from its own
//! `#[uses(…)]` and `#[use_type(…)]`, and a provider that omits one fails the provider trait's
//! `where` clause: an `E0277` on `__Context__`, at the `#[cgp_impl]` attribute or a method signature,
//! "required by a bound in `Greeter`".
//!
//! This module recovers the provider impl the error sits in and every requirement of its provider
//! trait that does not hold under the impl's own declarations, filling the rustc-free
//! [`UndeclaredRequirement`] the emitter words into a `[CGP-E036]` header.

use cargo_cgp_error_processing::{Requirement, UndeclaredRequirement};
use rustc_hir::{ImplItemKind, ItemKind};
use rustc_infer::infer::TyCtxtInferExt as _;
use rustc_infer::traits::Obligation;
use rustc_middle::ty::print::{PrintTraitRefExt as _, with_forced_trimmed_paths};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeFoldable as _, TypingMode};
use rustc_span::def_id::{DefId, LocalDefId};
use rustc_span::{Span, kw};
use rustc_trait_selection::traits::ObligationCause;
use rustc_trait_selection::traits::query::evaluate_obligation::InferCtxtExt as _;

use crate::resolve::query::{
    abstract_type_component_marker, context_param_span, is_provider_trait, local_trait_impls,
};

/// A component requirement a provider does not declare, with the impl and the traits that tie a
/// later report, or an abstract type reported on its own, to the same mistake.
pub struct DetectedUndeclaredRequirement {
    /// The provider trait, the provider, and the requirements, worded into the header and helps.
    pub requirement: UndeclaredRequirement,
    /// The provider impl.
    pub impl_did: DefId,
    /// The provider impl's full span, holding every use of the types it names.
    pub impl_span: Span,
    /// Each requirement's trait, in the order of [`UndeclaredRequirement::requirements`], and for an
    /// abstract-type component the name of the type it declares.
    pub traits: Vec<(DefId, Option<String>)>,
}

/// Recognize an `E0277` at `primary_span` as a requirement of a provider trait that a
/// `#[cgp_impl]` provider does not declare: the caret sits in a provider impl over the generated
/// `__Context__`, outside its method bodies (on the attribute, the impl header, or a signature),
/// and some `where` clause the provider trait places on the context does not hold under the impl's
/// own declarations. `None` otherwise.
pub fn detect_undeclared_requirement(
    tcx: TyCtxt<'_>,
    primary_span: Span,
) -> Option<DetectedUndeclaredRequirement> {
    for local in local_trait_impls(tcx) {
        let Some(macro_span) = context_param_span(tcx, local) else {
            continue;
        };
        // The caret is on the impl's header or a signature, or on the macro attribute that every
        // synthesized token shares.
        let item = tcx.hir_expect_item(local);
        let at_impl = item.span.contains(primary_span) || macro_span.overlaps(primary_span);
        if !at_impl || in_method_body(tcx, local, primary_span) {
            continue;
        }
        let impl_did = local.to_def_id();
        let trait_ref = tcx
            .impl_trait_ref(impl_did)
            .instantiate_identity()
            .skip_norm_wip();
        // A provider impl: a provider trait implemented for a provider type, over the context the
        // trait takes first.
        if !is_provider_trait(tcx, trait_ref.def_id)
            || matches!(trait_ref.self_ty().kind(), ty::Param(_))
        {
            continue;
        }
        let context = trait_ref.args.type_at(1);
        if !matches!(context.kind(), ty::Param(_)) {
            continue;
        }
        let param_env = tcx.param_env(impl_did);
        let mut requirements = Vec::new();
        let mut traits = Vec::new();
        for (clause, _) in tcx
            .clauses_of(trait_ref.def_id)
            .instantiate_own(tcx, trait_ref.args)
        {
            let clause = clause.skip_norm_wip();
            let Some(bound) = clause.as_trait_clause() else {
                continue;
            };
            let bound_ref = bound.skip_binder().trait_ref;
            if bound_ref.self_ty() != context || holds_in(tcx, param_env, clause) {
                continue;
            }
            let import_type = abstract_type_import(tcx, bound_ref.def_id);
            let rendered = with_forced_trimmed_paths!(
                context_as_self(tcx, bound_ref, context)
                    .print_only_trait_path()
                    .to_string()
            );
            requirements.push(Requirement {
                bound: rendered.clone(),
                import: import_type
                    .as_ref()
                    .map(|type_name| format!("{rendered}.{type_name}")),
            });
            traits.push((bound_ref.def_id, import_type));
        }
        if requirements.is_empty() {
            return None;
        }
        let provider = with_forced_trimmed_paths!(trait_ref.self_ty().to_string());
        return Some(DetectedUndeclaredRequirement {
            requirement: UndeclaredRequirement {
                provider_trait: tcx.item_name(trait_ref.def_id).to_string(),
                provider,
                requirements,
            },
            impl_did,
            impl_span: item.span,
            traits,
        });
    }
    None
}

/// `bound` with the generated context written as `Self`, the name the provider's `#[uses(…)]` gives
/// it, so a requirement such as `CanCompareWith<__Context__>` reads `CanCompareWith<Self>`.
fn context_as_self<'tcx>(
    tcx: TyCtxt<'tcx>,
    bound: ty::TraitRef<'tcx>,
    context: Ty<'tcx>,
) -> ty::TraitRef<'tcx> {
    let self_ty = Ty::new_param(tcx, 0, kw::SelfUpper);
    bound.fold_with(&mut ty::BottomUpFolder {
        tcx,
        ty_op: |ty| if ty == context { self_ty } else { ty },
        lt_op: |region| region,
        ct_op: |ct| ct,
    })
}

/// Whether `span` lies in the body of one of the impl's methods, where an error is about the body's
/// own code rather than about what the impl declares.
fn in_method_body(tcx: TyCtxt<'_>, impl_did: LocalDefId, span: Span) -> bool {
    let ItemKind::Impl(imp) = tcx.hir_expect_item(impl_did).kind else {
        return false;
    };
    imp.items
        .iter()
        .any(|&item_id| match tcx.hir_impl_item(item_id).kind {
            ImplItemKind::Fn(_, body) => tcx.hir_body(body).value.span.contains(span),
            _ => false,
        })
}

/// Whether `clause` holds under `param_env`, the impl's own declarations.
fn holds_in<'tcx>(
    tcx: TyCtxt<'tcx>,
    param_env: ty::ParamEnv<'tcx>,
    clause: ty::Clause<'tcx>,
) -> bool {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let obligation = Obligation::new(tcx, ObligationCause::dummy(), param_env, clause);
    infcx.predicate_must_hold_modulo_regions(&obligation)
}

/// The associated type an abstract-type component declares, which `#[use_type(Trait.Type)]`
/// imports, when `trait_did` is one ([`abstract_type_component_marker`]) and takes no parameter of
/// its own. `None` for any other trait, which `#[uses(…)]` declares.
fn abstract_type_import(tcx: TyCtxt<'_>, trait_did: DefId) -> Option<String> {
    if tcx.generics_of(trait_did).own_params.len() != 1 {
        return None;
    }
    abstract_type_component_marker(tcx, trait_did)?;
    let item = tcx
        .associated_items(trait_did)
        .in_definition_order()
        .next()?;
    Some(item.name().to_string())
}
