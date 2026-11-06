//! Looking a wiring name up: whether a type of the name is declared, and which component a key
//! names.

use cargo_cgp_error_processing::UnresolvedWiringName;
use rustc_hir::{QPath, TyKind};
use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;

use crate::config::COMPONENT_MARKER_SUFFIX;
use crate::detect::wiring_name::component::{
    consumer_component, consumer_trait_named, is_type_declared,
};
use crate::resolve::query::import_path;

/// A wiring key `name` that does not resolve: the one component of that name elsewhere, which needs
/// an import, or, when `name` is a consumer trait's name with `Component` appended, that trait's
/// component, named differently.
pub fn unresolved_key(
    tcx: TyCtxt<'_>,
    name: &str,
    markers: &[(DefId, DefId)],
) -> Option<UnresolvedWiringName> {
    if is_component_name(tcx, name, markers) {
        return component_not_imported(tcx, name, markers);
    }
    let consumer = name.strip_suffix(COMPONENT_MARKER_SUFFIX)?;
    let consumer_did = consumer_trait_named(tcx, consumer)?;
    let component_did = consumer_component(tcx, consumer_did)?;
    Some(UnresolvedWiringName::KeyNamesNoComponent {
        key: name.to_owned(),
        consumer: consumer.to_owned(),
        component: tcx.item_name(component_did).to_string(),
    })
}

/// The names of the providers that implement the provider trait of the component `key` names, when
/// it names one.
pub fn providers_of_key(
    tcx: TyCtxt<'_>,
    key: &rustc_hir::Ty<'_>,
    markers: &[(DefId, DefId)],
) -> Option<Vec<String>> {
    let TyKind::Path(QPath::Resolved(None, path)) = key.kind else {
        return None;
    };
    let key_did = path.res.opt_def_id()?;
    let &(_, provider_trait) = markers
        .iter()
        .find(|(marker_did, _)| *marker_did == key_did)?;
    let impls = tcx.trait_impls_of(provider_trait);
    Some(
        impls
            .non_blanket_impls()
            .values()
            .flatten()
            .filter_map(
                |&impl_did| match tcx.type_of(impl_did).skip_binder().kind() {
                    ty::Adt(provider_def, _) => Some(tcx.item_name(provider_def.did()).to_string()),
                    _ => None,
                },
            )
            .collect(),
    )
}

/// Whether some component marker is named `name`.
pub fn is_component_name(tcx: TyCtxt<'_>, name: &str, markers: &[(DefId, DefId)]) -> bool {
    markers
        .iter()
        .any(|(marker_did, _)| tcx.item_name(*marker_did).as_str() == name)
}

/// The component `name` names, when exactly one marker is so named, as one that needs an import.
/// `None` when no marker, several markers, or no public path to the one, so the import is
/// ambiguous.
pub fn component_not_imported(
    tcx: TyCtxt<'_>,
    name: &str,
    markers: &[(DefId, DefId)],
) -> Option<UnresolvedWiringName> {
    let mut named = markers
        .iter()
        .filter(|(marker_did, _)| tcx.item_name(*marker_did).as_str() == name);
    let &(marker_did, _) = named.next()?;
    if named.next().is_some() {
        return None;
    }
    Some(UnresolvedWiringName::ComponentNotImported {
        component: name.to_owned(),
        path: import_path(tcx, marker_did)?,
    })
}

/// Whether a type of each name asked about is declared where the crate could import it from,
/// remembered per name, since the answer walks the crate and its dependencies.
#[derive(Default)]
pub struct DeclaredNames(Vec<(String, bool)>);

impl DeclaredNames {
    /// Whether a type named `name` is declared where the crate could import it from, computed on
    /// the first ask for that name and remembered.
    pub fn is_declared(&mut self, tcx: TyCtxt<'_>, name: &str) -> bool {
        if let Some(&(_, declared)) = self.0.iter().find(|(known, _)| known == name) {
            return declared;
        }
        let declared = is_type_declared(tcx, name);
        self.0.push((name.to_owned(), declared));
        declared
    }
}
