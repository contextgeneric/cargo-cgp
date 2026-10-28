//! Detecting an abstract type a CGP definition uses but does not import.
//!
//! `Self::Error` in a `#[cgp_fn]`, `#[cgp_impl]`, or `#[cgp_component]` definition names an
//! associated type of the generated context (`Self`, or the `__Context__` `#[cgp_impl]` writes for
//! it). Without `#[use_type(HasErrorType.Error)]` that context carries no bound declaring one, and
//! the compiler reports `E0220`, "associated type `Error` not found for `__Context__`". The fix is
//! the import, so this module recovers which abstract-type trait declares the name and fills the
//! rustc-free [`UnimportedAbstractType`] the emitter words into a `[CGP-E035]` header.
//!
//! The `E0220` arrives while the compiler lowers the definition's types, so only the HIR and the
//! trait definitions are read: the definition the caret sits in is recognized from the generated
//! items' fingerprints, never by forcing the signature or body being lowered.

use cargo_cgp_error_processing::UnimportedAbstractType;
use rustc_hir::def::DefKind;
use rustc_hir::{GenericParamKind, ItemKind, ParamName};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::{DefId, LocalDefId};
use rustc_span::{Span, Symbol};

use crate::config::CGP_IMPL_CONTEXT_PARAM;
use crate::resolve::cgp_item::{
    abstract_type_component_marker, is_consumer_trait, is_provider_trait,
};

/// An abstract type a CGP definition uses but does not import, with the items that recognize a
/// later report of the same mistake.
pub struct DetectedUnimportedAbstractType {
    /// The abstract type and its trait, worded into the header and the help.
    pub unimported: UnimportedAbstractType,
    /// The generated impl or trait the use sits in.
    pub item: DefId,
    /// The abstract-type trait that declares the type.
    pub trait_did: DefId,
}

/// Recognize an `E0220` at `primary_span` for the associated type `type_name` of `qself` as an
/// abstract type a CGP definition uses without importing it. `None` unless `qself` is the generated
/// context (`Self` or `__Context__`), the caret sits in an item a CGP macro generated, and exactly
/// one CGP abstract-type component declares an associated type of that name, which is then the one
/// to import. A second such trait makes the import ambiguous, so the error is left as rustc wrote
/// it.
pub fn detect_unimported_abstract_type(
    tcx: TyCtxt<'_>,
    primary_span: Span,
    type_name: &str,
    qself: &str,
) -> Option<DetectedUnimportedAbstractType> {
    if qself != "Self" && qself != CGP_IMPL_CONTEXT_PARAM {
        return None;
    }
    let item = enclosing_cgp_item(tcx, primary_span)?;
    let trait_did = abstract_type_trait(tcx, Symbol::intern(type_name))?;
    Some(DetectedUnimportedAbstractType {
        unimported: UnimportedAbstractType {
            type_name: type_name.to_owned(),
            trait_name: tcx.item_name(trait_did).to_string(),
        },
        item,
        trait_did,
    })
}

/// The impl or trait a CGP macro generated whose method (signature or body) holds `span`: an impl
/// over the `__Context__` parameter `#[cgp_impl]`, `#[cgp_fn]`, and `#[cgp_component]` give their
/// generated impls, a consumer or provider trait, or a `#[cgp_fn]` trait, recognized by its blanket
/// impl over that parameter.
pub fn enclosing_cgp_item(tcx: TyCtxt<'_>, span: Span) -> Option<DefId> {
    let items = tcx.hir_crate_items(());
    let methods = items
        .trait_items()
        .map(|id| id.owner_id.def_id)
        .chain(items.impl_items().map(|id| id.owner_id.def_id));
    for method in methods {
        if tcx.def_kind(method) != DefKind::AssocFn
            || !tcx
                .hir_span_with_body(tcx.local_def_id_to_hir_id(method))
                .contains(span)
        {
            continue;
        }
        let parent = tcx.local_parent(method);
        let generated = match tcx.def_kind(parent) {
            DefKind::Impl { .. } => has_context_param(tcx, parent),
            DefKind::Trait => {
                let trait_did = parent.to_def_id();
                is_consumer_trait(tcx, trait_did)
                    || is_provider_trait(tcx, trait_did)
                    || tcx
                        .trait_impls_of(trait_did)
                        .blanket_impls()
                        .iter()
                        .filter_map(|impl_did| impl_did.as_local())
                        .any(|impl_did| has_context_param(tcx, impl_did))
            }
            _ => false,
        };
        if generated {
            return Some(parent.to_def_id());
        }
    }
    None
}

/// Whether the impl declares a type parameter named `__Context__`, the reserved context the CGP
/// macros give their generated impls. Read from the HIR, so no query on the impl is forced.
pub fn has_context_param(tcx: TyCtxt<'_>, impl_did: LocalDefId) -> bool {
    context_param_span(tcx, impl_did).is_some()
}

/// The span of the impl's `__Context__` type parameter, the macro call site every token a CGP macro
/// synthesizes shares, or `None` when the impl declares no such parameter.
pub fn context_param_span(tcx: TyCtxt<'_>, impl_did: LocalDefId) -> Option<Span> {
    let ItemKind::Impl(imp) = tcx.hir_expect_item(impl_did).kind else {
        return None;
    };
    imp.generics.params.iter().find_map(|param| {
        (matches!(param.kind, GenericParamKind::Type { .. })
            && matches!(param.name, ParamName::Plain(ident) if ident.name.as_str() == CGP_IMPL_CONTEXT_PARAM))
        .then_some(param.span)
    })
}

/// The one CGP abstract-type component ([`abstract_type_component_marker`]) that declares an
/// associated type named `name` and takes no parameter of its own, so `#[use_type(Trait.Name)]`
/// writes it in full. `None` when there is none, or more than one.
pub fn abstract_type_trait(tcx: TyCtxt<'_>, name: Symbol) -> Option<DefId> {
    let mut found = None;
    for trait_did in tcx.all_traits_including_private() {
        let declares = tcx
            .associated_items(trait_did)
            .filter_by_name_unhygienic(name)
            .any(|item| item.kind.tag() == ty::AssocTag::Type);
        if !declares
            || tcx.generics_of(trait_did).own_params.len() != 1
            || abstract_type_component_marker(tcx, trait_did).is_none()
        {
            continue;
        }
        if found.replace(trait_did).is_some() {
            return None;
        }
    }
    found
}
