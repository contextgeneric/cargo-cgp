//! The values every anchor and walk step takes together: the compiler handle and the cache.
//!
//! See `cgp-knowledge-base/cargo-cgp/implementation/architecture.md`. A `TyCtxt` exists only
//! inside the emitter's `ty::tls::with_opt` closure, while the cache lives on the emitter for the
//! whole compilation, so the context is built per diagnostic inside that closure and borrows the
//! cache rather than owning it.

use rustc_middle::ty::TyCtxt;

use crate::resolve::cache::ResolveCache;

/// The compiler handle and the per-compilation [`ResolveCache`], threaded as one value through the
/// anchors and the walk. A detector that never reads the cache takes the `TyCtxt` alone.
#[derive(Clone, Copy)]
pub struct ResolveCtx<'a, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub cache: &'a ResolveCache,
}
