//! What each of two colliding entries maps the colliding key to.

use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;

use crate::detect::conflict::namespace_delegate;
use crate::resolve::query::{redirect_path, render_provider};

/// One side of a conflict: the impl, the key it maps, and its written `Delegate`.
pub struct ConflictSide<'tcx> {
    pub impl_did: DefId,
    pub key: Ty<'tcx>,
    pub delegate: Option<Ty<'tcx>>,
}

/// What an entry maps the colliding key to: a redirected path, or a provider a reader wrote.
pub enum EntryTarget {
    Redirect(String),
    Provider(Option<String>),
}

impl<'tcx> ConflictSide<'tcx> {
    /// Whether the entry is a blanket forwarding over every key (a namespace join, an inheritance
    /// blanket, or a `for` loop), whose key is a generic parameter.
    pub fn is_blanket(&self) -> bool {
        matches!(self.key.kind(), ty::Param(_))
    }

    /// What this entry maps `other_key` to. A blanket forwarding's own `Delegate` is a generic
    /// projection, so its target is read by resolving the forwarding for `other_key`; an ordinary
    /// entry's target is its written `Delegate`. A provider that still names a generic parameter
    /// is not one a reader wrote, so it is left unnamed.
    pub fn target(&self, tcx: TyCtxt<'tcx>, other_key: Ty<'tcx>) -> EntryTarget {
        let delegate = if self.is_blanket() {
            namespace_delegate(tcx, self.impl_did, self.key, other_key)
        } else {
            self.delegate
        };
        match delegate {
            Some(delegate) => match redirect_path(tcx, delegate) {
                Some(path) => EntryTarget::Redirect(path),
                None => EntryTarget::Provider(
                    (!delegate.has_param()).then(|| render_provider(tcx, delegate)),
                ),
            },
            None => EntryTarget::Provider(None),
        }
    }
}
