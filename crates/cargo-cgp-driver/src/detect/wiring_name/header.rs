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
use rustc_hir::{GenericArg, ItemKind, Path, UseKind, UseTree};
use rustc_middle::ty::{self, GenericParamDefKind, TyCtxt};
use rustc_span::def_id::{DefId, LocalDefId, LocalModId};
use rustc_span::{BytePos, Span};

use crate::config::COMPONENT_MARKER_SUFFIX;
use crate::detect::wiring_name::{NameFix, import_edit};
use crate::resolve::query::{
    has_one_import_path, import_path, local_trait_impls, provider_blanket_marker, same_range,
};

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
    let component = attribute_component(tcx, header);
    Some(UnresolvedWiringName::DerivedComponentName {
        derived: header.derived(),
        written: header.written(),
        provider_trait: tcx.item_name(header.provider_trait).to_string(),
        component: declared,
        attribute: attribute_naming(tcx, header.impl_did, &component)?.text,
        imports_component: module_imports(tcx, module_of(tcx, header), header.marker, false),
    })
}

/// The fix for `name`, the header's derived name that is not its component's, at `name_span`: the
/// component added to the header's attribute, after the provider of `#[cgp_impl(...)]` or as the
/// argument of a provider attribute. A key not in scope at the header is imported by the same fix,
/// with the `use` line a missing component's import inserts, or written by its path when the module
/// has no place for that line. The fix is machine-applicable when the header writes an argument
/// for each of the key's parameters and one path imports a key it imports, and only maybe correct
/// otherwise, as when a parameter's name stands in for an argument. `None` when the attribute is
/// not where the source can be edited.
pub fn derived_component_fix(
    tcx: TyCtxt<'_>,
    header: &ProviderHeader<'_>,
    name: &UnresolvedWiringName,
    name_span: Span,
) -> Option<NameFix> {
    let (args, all_written) = attribute_args(tcx, header);
    let marker_name = tcx.item_name(header.marker).to_string();
    let module = module_of(tcx, header);
    let naming = |key: &str| {
        attribute_naming(tcx, header.impl_did, &with_args(key, &args))
            .and_then(|naming| naming.insertion)
    };
    if key_in_scope(tcx, module, header.marker) {
        return NameFix::new(
            tcx,
            name,
            false,
            vec![naming(&marker_name)?],
            name_span,
            all_written,
        );
    }
    let path = import_path(tcx, header.marker)?;
    match import_edit(tcx, module, &path) {
        Some(import) => {
            let one_path = has_one_import_path(tcx, header.marker, module.to_local_def_id());
            let edits = vec![import, naming(&marker_name)?];
            NameFix::new(tcx, name, true, edits, name_span, all_written && one_path)
        }
        None => NameFix::new(
            tcx,
            name,
            false,
            vec![naming(&path)?],
            name_span,
            all_written,
        ),
    }
}

/// The component as the help names it in the header's attribute, with the header's arguments for
/// its parameters: bare when it is in scope at the header, and by its import path otherwise, so
/// the attribute resolves where the header is.
fn attribute_component(tcx: TyCtxt<'_>, header: &ProviderHeader<'_>) -> String {
    let marker_name = tcx.item_name(header.marker).to_string();
    let (args, _) = attribute_args(tcx, header);
    let module = module_of(tcx, header);
    let key = if key_in_scope(tcx, module, header.marker) {
        marker_name
    } else {
        import_path(tcx, header.marker).unwrap_or(marker_name)
    };
    with_args(&key, &args)
}

/// The arguments the header's attribute gives the component's parameters, the header's own at
/// each parameter's position, and whether the header writes each of them. A parameter the header
/// writes no argument for keeps its name.
fn attribute_args(tcx: TyCtxt<'_>, header: &ProviderHeader<'_>) -> (Vec<String>, bool) {
    let params = own_param_names(tcx, header.marker);
    let written: Vec<Option<String>> = params
        .iter()
        .map(|param| header_arg_for(tcx, header, param))
        .collect();
    let all_written = written.iter().all(Option::is_some);
    let args = written
        .into_iter()
        .zip(&params)
        .map(|(arg, param)| arg.unwrap_or_else(|| param.clone()))
        .collect();
    (args, all_written)
}

/// Whether `module` names `marker` unqualified: the module declares it, or a `use` item of the
/// module imports it by name or through a glob of the module that declares or re-exports it.
fn key_in_scope(tcx: TyCtxt<'_>, module: LocalModId, marker: DefId) -> bool {
    marker
        .as_local()
        .is_some_and(|local| tcx.parent_module_from_def_id(local) == module)
        || module_imports(tcx, module, marker, true)
}

/// The module holding the header's impl.
fn module_of(tcx: TyCtxt<'_>, header: &ProviderHeader<'_>) -> LocalModId {
    tcx.parent_module_from_def_id(header.impl_did)
}

/// Whether a `use` item of `module` imports `marker` by name, or, with `globs`, also through a
/// glob of a module that declares or re-exports it.
fn module_imports(tcx: TyCtxt<'_>, module: LocalModId, marker: DefId, globs: bool) -> bool {
    tcx.hir_get_module(module)
        .0
        .item_ids
        .iter()
        .any(|&item_id| {
            let ItemKind::Use(tree) = tcx.hir_item(item_id).kind else {
                return false;
            };
            use_tree_imports(tcx, &tree, marker, globs)
        })
}

/// Whether the `use` tree imports `marker` by name, or, with `globs`, through a glob of a module
/// that declares or re-exports it, searching each list `{a, b}` the tree holds.
fn use_tree_imports(tcx: TyCtxt<'_>, tree: &UseTree<'_>, marker: DefId, globs: bool) -> bool {
    if let UseKind::Nested { items } = tree.kind {
        return items
            .iter()
            .any(|(item, ..)| use_tree_imports(tcx, item, marker, globs));
    }
    let Some(Res::Def(kind, did)) = tree.prefix.res.type_ns else {
        return false;
    };
    match tree.kind {
        UseKind::Single(_) => did == marker,
        UseKind::Glob if globs && kind == DefKind::Mod => {
            let children = match did.as_local() {
                Some(local) => tcx.module_children_local(local),
                None => tcx.module_children(did),
            };
            children
                .iter()
                .any(|child| child.res.opt_def_id() == Some(marker))
        }
        UseKind::Glob | UseKind::Nested { .. } => false,
    }
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

/// The attribute of a header written to name its component, and the edit that writes it.
struct AttributeNaming {
    /// The attribute with the component named.
    text: String,
    /// Where the component goes in the attribute as written, and the text that adds it there, when
    /// the attribute is found in the source.
    insertion: Option<(Span, String)>,
}

/// The attribute of the header at `impl_did`, rewritten to name `component` explicitly, in the
/// spelling the program gave the macro. A `#[cgp_impl]` block writes its provider, the impl's `Self`
/// type, inside the attribute, before the trait's name, so the attribute's text up to the provider
/// is kept and the component appended: `#[cgp_impl(new GreetHello: GreeterComponent)]`. An impl
/// that writes its provider after `for` takes the component as the argument of its provider
/// attribute, the nearest attribute above the impl whose macro name ends in `provider`:
/// `#[cgp_provider(GreeterComponent)]` or `#[cgp_new_provider(GreeterComponent)]`.
fn attribute_naming(
    tcx: TyCtxt<'_>,
    impl_did: LocalDefId,
    component: &str,
) -> Option<AttributeNaming> {
    let ItemKind::Impl(imp) = tcx.hir_expect_item(impl_did).kind else {
        return None;
    };
    let source_map = tcx.sess.source_map();
    let trait_name = imp.of_trait?.trait_ref.path.segments.last()?.ident.span;
    if imp.self_ty.span.lo() < trait_name.lo() {
        let provider = source_map.span_to_snippet(imp.self_ty.span).ok()?;
        let before_provider = source_map.span_to_prev_source(imp.self_ty.span).ok()?;
        let opening = &before_provider[before_provider.rfind("#[")?..];
        return Some(AttributeNaming {
            text: format!("{opening}{provider}: {component})]"),
            insertion: Some((imp.self_ty.span.shrink_to_hi(), format!(": {component}"))),
        });
    }
    let impl_span = tcx.def_span(impl_did);
    let before_impl = source_map.span_to_prev_source(impl_span).ok()?;
    let found = before_impl.rmatch_indices("#[").find_map(|(start, _)| {
        let attribute = &before_impl[start + 2..];
        let path = attribute.split(['(', ']']).next()?;
        let macro_path = path.trim();
        macro_path
            .ends_with("provider")
            .then_some((start + 2 + path.len(), macro_path))
    });
    let macro_path = found.map_or("cgp_provider", |(_, macro_path)| macro_path);
    let insertion = found.and_then(|(path_end, _)| {
        let after = &before_impl[path_end..];
        let (offset, text) = if after.starts_with(']') {
            (path_end, format!("({component})"))
        } else if let Some(inside) = after.strip_prefix('(')
            && inside.split(')').next()?.trim().is_empty()
        {
            (path_end + 1, component.to_owned())
        } else {
            return None;
        };
        let back = u32::try_from(before_impl.len() - offset).ok()?;
        let at = impl_span
            .with_lo(impl_span.lo() - BytePos(back))
            .shrink_to_lo();
        Some((at, text))
    });
    Some(AttributeNaming {
        text: format!("#[{macro_path}({component})]"),
        insertion,
    })
}
