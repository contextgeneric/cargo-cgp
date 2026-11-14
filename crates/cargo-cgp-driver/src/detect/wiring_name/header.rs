//! Recognizing the component a `#[cgp_impl]` or `#[cgp_provider]` header derives, when it does not
//! resolve.
//!
//! The macros derive a provider's component from its provider trait's path as written, appending
//! `Component` to the last segment and spanning the key on that segment's name, so a key that does
//! not resolve is reported on the trait's name in the header. The header's own trait reference
//! resolves, so the provider trait, and through its delegation blanket the real component, is read
//! by `DefId`, never by name: two components whose provider traits share a name stay distinct.

use cargo_cgp_error_processing::UnresolvedWiringName;
use rustc_hir::def::{DefKind, Res};
use rustc_hir::{GenericArg, ItemKind, Path};
use rustc_middle::ty::{self, GenericParamDefKind, TyCtxt};
use rustc_span::Span;
use rustc_span::def_id::{DefId, LocalDefId};

use crate::config::COMPONENT_MARKER_SUFFIX;
use crate::resolve::query::{import_path, local_trait_impls, provider_blanket_marker, same_range};

/// A provider header whose trait name sits at a span: the impl, its trait path as written, the
/// provider trait it resolves to, and the component that trait keys on.
pub struct ProviderHeader<'tcx> {
    /// The impl the header heads.
    pub impl_did: LocalDefId,
    /// The trait path as written, its last segment the name the macro derives the component from.
    pub path: &'tcx Path<'tcx>,
    /// The provider trait the path resolves to.
    pub provider_trait: DefId,
    /// The component marker the provider trait keys on.
    pub marker: DefId,
}

impl ProviderHeader<'_> {
    /// The trait name the header writes, the path's last segment.
    pub fn written(&self) -> String {
        self.path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default()
    }

    /// The component the macro derives from the written name, `{written}Component`.
    pub fn derived(&self) -> String {
        format!("{}{COMPONENT_MARKER_SUFFIX}", self.written())
    }
}

/// The local impl of a provider trait whose trait path's last segment is the name at `span`: the
/// header a component key derived there belongs to. A trait that is no provider trait, such as a
/// consumer trait or an ordinary trait named in a `#[cgp_impl]` header, is no such header.
pub fn provider_header_at(tcx: TyCtxt<'_>, span: Span) -> Option<ProviderHeader<'_>> {
    local_trait_impls(tcx).find_map(|impl_did| {
        let ItemKind::Impl(imp) = tcx.hir_expect_item(impl_did).kind else {
            return None;
        };
        let path = imp.of_trait?.trait_ref.path;
        let last = path.segments.last()?;
        if !same_range(last.ident.span, span) {
            return None;
        }
        let Res::Def(DefKind::Trait, provider_trait) = path.res else {
            return None;
        };
        let marker = provider_blanket_marker(tcx, provider_trait)?;
        let ty::Adt(marker_def, _) = marker.kind() else {
            return None;
        };
        Some(ProviderHeader {
            impl_did,
            path,
            provider_trait,
            marker: marker_def.did(),
        })
    })
}

/// The mistake behind a component the header derives that names nothing: `name`, from rustc's
/// message, must be the derived name, and `module` is the module rustc says lacks it when the
/// header names the trait by a path. The real component taking that name means it is not imported,
/// or not re-exported by `module`; another name means the header derived the wrong one.
pub fn header_component_mistake(
    tcx: TyCtxt<'_>,
    header: &ProviderHeader<'_>,
    name: &str,
    module: Option<&str>,
) -> Option<UnresolvedWiringName> {
    if name != header.derived() {
        return None;
    }
    if tcx.item_name(header.marker).as_str() != name || has_own_params(tcx, header.marker) {
        return derived_component_name(tcx, header);
    }
    let path = import_path(tcx, header.marker)?;
    match module {
        None => Some(UnresolvedWiringName::ComponentNotImported {
            component: name.to_owned(),
            path,
        }),
        Some(module) => {
            let (defining_module, _) = path.rsplit_once("::")?;
            Some(UnresolvedWiringName::ComponentNotInModule {
                component: name.to_owned(),
                module: module.to_owned(),
                trait_path: format!(
                    "{defining_module}::{}",
                    tcx.item_name(header.provider_trait)
                ),
            })
        }
    }
}

/// The header derived a name that is not its component's: the trait is imported under another name,
/// or the component's `name:` renames its key or gives it parameters. Words the real component and
/// the header's attribute naming it.
pub fn derived_component_name(
    tcx: TyCtxt<'_>,
    header: &ProviderHeader<'_>,
) -> Option<UnresolvedWiringName> {
    let marker_name = tcx.item_name(header.marker);
    let params = own_param_names(tcx, header.marker);
    let declared = with_args(marker_name.as_str(), &params);
    let args: Vec<String> = params
        .iter()
        .map(|param| header_arg_for(tcx, header, param).unwrap_or_else(|| param.clone()))
        .collect();
    let component = with_args(marker_name.as_str(), &args);
    Some(UnresolvedWiringName::DerivedComponentName {
        derived: header.derived(),
        written: header.written(),
        provider_trait: tcx.item_name(header.provider_trait).to_string(),
        component: declared,
        attribute: attribute_naming(tcx, header.impl_did, &component)?,
    })
}

/// Whether the item declares generic parameters of its own.
fn has_own_params(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    !tcx.generics_of(def_id).own_params.is_empty()
}

/// The names of the item's own generic parameters, a lifetime with its `'`.
fn own_param_names(tcx: TyCtxt<'_>, def_id: DefId) -> Vec<String> {
    tcx.generics_of(def_id)
        .own_params
        .iter()
        .map(|param| param.name.to_string())
        .collect()
}

/// `name` with its arguments, `name<A, B>`, or `name` alone without any.
fn with_args(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        name.to_owned()
    } else {
        format!("{name}<{}>", args.join(", "))
    }
}

/// The argument the header writes for the provider trait's parameter `param`, as the source spells
/// it. A component's key takes the trait's own parameters by name, so the header's argument at the
/// same position is the one the key takes. `None` when the trait has no such parameter or the
/// header writes its arguments differently, such as with an elided lifetime.
fn header_arg_for(tcx: TyCtxt<'_>, header: &ProviderHeader<'_>, param: &str) -> Option<String> {
    let generics = tcx.generics_of(header.provider_trait);
    let trait_params: Vec<_> = generics
        .own_params
        .iter()
        .filter(|param| !matches!(param.kind, GenericParamDefKind::Type { .. } if param.index == 0 && generics.has_self))
        .collect();
    let args = header.path.segments.last()?.args?.args;
    if args.len() != trait_params.len() {
        return None;
    }
    let position = trait_params
        .iter()
        .position(|trait_param| trait_param.name.as_str() == param)?;
    let span = match &args[position] {
        GenericArg::Type(ty) => ty.span,
        GenericArg::Lifetime(lifetime) => lifetime.ident.span,
        GenericArg::Const(constant) => constant.span,
        GenericArg::Infer(infer) => infer.span,
    };
    tcx.sess.source_map().span_to_snippet(span).ok()
}

/// The attribute of the header at `impl_did`, rewritten to name `component` explicitly:
/// `#[cgp_impl(new GreetHello: GreeterComponent)]` for a `#[cgp_impl]` block, whose provider is the
/// impl's `Self` type written inside the attribute, and `#[cgp_provider(GreeterComponent)]` or
/// `#[cgp_new_provider(GreeterComponent)]` for an impl that writes its provider after `for`.
fn attribute_naming(tcx: TyCtxt<'_>, impl_did: LocalDefId, component: &str) -> Option<String> {
    let ItemKind::Impl(imp) = tcx.hir_expect_item(impl_did).kind else {
        return None;
    };
    let source_map = tcx.sess.source_map();
    let provider = source_map.span_to_snippet(imp.self_ty.span).ok()?;
    let before_provider = source_map.span_to_prev_source(imp.self_ty.span).ok()?;
    let before_provider = before_provider.trim_end();
    let (new, before_new) = match before_provider.strip_suffix("new") {
        Some(rest) => ("new ", rest.trim_end()),
        None => ("", before_provider),
    };
    if before_new.ends_with("cgp_impl(") {
        return Some(format!("#[cgp_impl({new}{provider}: {component})]"));
    }
    let before_impl = source_map
        .span_to_prev_source(tcx.def_span(impl_did))
        .ok()?;
    let attribute_line = before_impl.trim_end().lines().last().unwrap_or_default();
    let macro_name = if attribute_line.contains("cgp_new_provider") {
        "cgp_new_provider"
    } else {
        "cgp_provider"
    };
    Some(format!("#[{macro_name}({component})]"))
}
