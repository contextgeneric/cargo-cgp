//! Whether the crate being compiled uses CGP, and whether an item comes from one of CGP's crates.

use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;

use crate::config::{CGP_CRATES, CGP_UMBRELLA_CRATE};

/// Whether the crate being compiled uses `cgp` or one of its own crates ([`CGP_CRATES`]): the
/// compiler-side evidence that a diagnostic it emits can carry a CGP encoding at all. A crate that
/// declares `cgp` as a dependency but never names it does not use it, so its own types that merely
/// share CGP's names are never resugared.
///
/// `None` while the crate store can still change, during name resolution: the crate-list queries
/// freeze the store, and freezing it under the resolver makes the compiler panic when the resolver
/// next loads a crate. The store is frozen once anything has listed the crates, which the trait
/// solver does when it first looks up the impls of a trait from another crate (`trait_impls_of`),
/// so before any error about a CGP trait.
pub fn crate_uses_cgp(tcx: TyCtxt<'_>) -> Option<bool> {
    if !tcx.untracked().cstore.is_frozen() {
        return None;
    }
    Some(links_cgp(tcx))
}

/// Whether the crate links CGP, the answer [`crate_uses_cgp`] gives once the crate store is safe to
/// freeze. Call it only when name resolution is over, such as from the `after_expansion` callback.
pub fn links_cgp(tcx: TyCtxt<'_>) -> bool {
    tcx.used_crates(()).iter().any(|&krate| {
        let name = tcx.crate_name(krate);
        name.as_str() == CGP_UMBRELLA_CRATE || CGP_CRATES.contains(&name.as_str())
    })
}

/// Whether `def_id` is defined by one of CGP's own crates ([`CGP_CRATES`]), anchored to the
/// defining crate, so a same-named trait elsewhere never counts as evidence.
pub fn is_cgp_crate_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    let krate = tcx.crate_name(def_id.krate);
    CGP_CRATES.iter().any(|name| krate.as_str() == *name)
}
