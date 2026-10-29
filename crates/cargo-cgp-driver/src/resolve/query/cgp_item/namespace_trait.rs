//! Recognizing a namespace lookup trait by its shape and its evidence of CGP.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::def_id::DefId;

use crate::resolve::query::{is_cgp_crate_trait, is_component_marker, is_path_cons};

/// Whether `def_id` is a **namespace lookup trait** — the fingerprint every `cgp_namespace!`
/// (and the built-in `DefaultNamespace`/`DefaultImpls*`) generates: a trait whose *only*
/// associated item is a type named `Delegate`, resolving a key to its delegate provider. These are
/// recognized by that structural signature rather than by name or crate, because a user's own
/// namespace trait (`MockNamespace`) lives in the user crate and shares no `DefId` with CGP's, yet
/// must not be mistaken for a regular Rust trait: a failing namespace-lookup bound means the
/// redirect path is unwired, not that some ordinary bound is unmet.
///
/// The shape alone is not enough, since any user trait can have a lone `type Delegate`, so the
/// trait must also show it is a namespace ([`namespace_has_cgp_evidence`]).
pub fn is_namespace_lookup_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    has_namespace_shape(tcx, def_id) && namespace_has_cgp_evidence(tcx, def_id, 0)
}

/// Whether `def_id` is a trait whose only associated item is a type named `Delegate`.
pub fn has_namespace_shape(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    if !tcx.is_trait(def_id) {
        return false;
    }
    let mut items = tcx.associated_items(def_id).in_definition_order();
    let Some(first) = items.next() else {
        return false;
    };
    items.next().is_none()
        && first.kind.tag() == ty::AssocTag::Type
        && tcx.item_name(first.def_id).as_str() == "Delegate"
}

/// How far the namespace-evidence search follows a namespace to the parent it inherits. A real
/// hierarchy is a few levels deep, and the bound also stops a cycle of namespaces from looping.
pub const MAX_NAMESPACE_DEPTH: u32 = 8;

/// Whether a namespace-shaped trait shows that CGP generated it: it is one of CGP's own
/// ([`CGP_CRATES`](crate::config::CGP_CRATES), such as `DefaultNamespace`), an impl of it is keyed
/// by a component marker or a `PathCons` path (the entries `cgp_namespace!` writes), or an impl's
/// `where` clause names a parent namespace that shows the same evidence (the inheritance blanket
/// every child namespace gets).
pub fn namespace_has_cgp_evidence(tcx: TyCtxt<'_>, def_id: DefId, depth: u32) -> bool {
    if is_cgp_crate_trait(tcx, def_id) {
        return true;
    }
    if depth >= MAX_NAMESPACE_DEPTH {
        return false;
    }
    tcx.all_impls(def_id).any(|impl_did| {
        let key = tcx.impl_trait_ref(impl_did).skip_binder().self_ty();
        is_path_cons(tcx, key)
            || is_component_marker(tcx, key)
            || tcx
                .clauses_of(impl_did)
                .clauses
                .iter()
                .filter_map(|(clause, _)| clause.as_trait_clause())
                .any(|bound| {
                    let parent = bound.def_id();
                    parent != def_id
                        && has_namespace_shape(tcx, parent)
                        && namespace_has_cgp_evidence(tcx, parent, depth + 1)
                })
    })
}
