//! Finding the trait impl a definition sits in.

use rustc_hir::def::DefKind;
use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;

/// The nearest enclosing trait impl of `did`, walking up the def-parent chain (through the
/// intervening `#[async_trait]` coroutine body and its method), or `None` if none is reached before
/// the crate root. This is what lets the detection work uniformly for a sync `#[cgp_fn]` body (impl
/// one hop up) and an async one (impl two hops up, past the async-block coroutine).
pub fn enclosing_trait_impl(tcx: TyCtxt<'_>, mut did: DefId) -> Option<DefId> {
    loop {
        if matches!(tcx.def_kind(did), DefKind::Impl { of_trait: true }) {
            return Some(did);
        }
        let parent = tcx.opt_parent(did)?;
        if parent == did {
            return None;
        }
        did = parent;
    }
}
