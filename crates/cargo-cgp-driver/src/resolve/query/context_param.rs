//! Reading the reserved `__Context__` parameter the CGP macros give their generated impls.

use cargo_cgp_error_processing::invokes_macro;
use rustc_hir::{GenericParamKind, ItemKind, ParamName};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::{DefId, LocalDefId};
use rustc_span::hygiene::{ExpnKind, MacroKind};

use crate::config::{CGP_FN_MACRO, CGP_IMPL_CONTEXT_PARAM, CGP_IMPL_MACRO};

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

/// Whether `impl_did` is the impl a `#[cgp_fn]` or `#[cgp_impl]` lowered a definition into, the
/// definitions that take `#[uses]` and `#[use_provider]`. `#[cgp_impl]` writes the impl's trait
/// path, in its explicit-context form too, which names its context itself, and `#[cgp_fn]` the
/// whole item; a hand-written impl, with or without `#[cgp_provider]`, keeps the spans its program
/// wrote.
pub fn is_cgp_definition_impl(tcx: TyCtxt<'_>, impl_did: DefId) -> bool {
    let Some(local) = impl_did.as_local() else {
        return false;
    };
    let item = tcx.hir_expect_item(local);
    let ItemKind::Impl(imp) = item.kind else {
        return false;
    };
    let from_macro = |span: Span, name: &str| {
        span.macro_backtrace().any(|expansion| {
            matches!(expansion.kind, ExpnKind::Macro(MacroKind::Attr, macro_name)
                if invokes_macro(macro_name.as_str(), name))
        })
    };
    from_macro(item.span, CGP_FN_MACRO)
        || imp
            .of_trait
            .is_some_and(|header| from_macro(header.trait_ref.path.span, CGP_IMPL_MACRO))
}
