//! Reading the reserved `__Context__` parameter the CGP macros give their generated impls.

use rustc_hir::{GenericParamKind, ItemKind, ParamName};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use rustc_span::def_id::LocalDefId;

use crate::config::CGP_IMPL_CONTEXT_PARAM;

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
