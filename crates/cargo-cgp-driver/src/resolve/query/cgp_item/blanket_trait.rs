//! Recognizing a `#[cgp_fn]` or `#[blanket_trait]` blanket trait by its blanket's evidence of CGP.

use rustc_middle::ty::TyCtxt;
use rustc_span::def_id::DefId;

use crate::config::CGP_CRATES;
use crate::resolve::query::{is_cgp_crate_trait, is_consumer_trait};

/// Whether `def_id` is a **blanket trait** — one carrying a blanket impl over a bare context
/// parameter (`impl<Context> Trait for Context where Self: …`). This is the shape `#[cgp_fn]` and
/// `#[blanket_trait]` generate, and the hand-written desugaring of them: the trait is available to
/// any context meeting the blanket's `where` bounds (a `HasField`, another such trait), so a
/// failing `Ctx: Trait` bound has a recoverable root cause down those bounds even though the trait
/// is not a CGP *component* (it has no provider trait or `DelegateComponent`). A CGP consumer trait
/// also matches (its consumer blanket is a blanket impl), which is harmless: callers that care about
/// the distinction test
/// [`consumer_provider_trait`](crate::resolve::query::consumer_provider_trait) separately.
///
/// The blanket impl alone is far too broad a signal to accept, since `ToString`, `Into`, and
/// `Borrow` all carry one, and so does any ordinary blanket trait a CGP crate defines for itself
/// (`impl<T: Clone> Describe for T`). Reshaping their failures into CGP errors would be an
/// over-reach, so a trait qualifies only when its blanket genuinely depends on CGP
/// ([`blanket_depends_on_cgp`]), wherever the trait is defined. That admits the `#[cgp_fn]` output
/// of the checked crate and a trait a library publishes alike.
///
/// A blanket trait one of CGP's own crates defines ([`CGP_CRATES`]) is excluded: those are the
/// extensible-data casts and builders (`CanUpcast`, `CanBuildFrom`), whose failures end on the
/// extractor and builder recursions the walk cannot yet word as the field or variant at fault,
/// while the compiler's own headline names it (`Big: FromVariant<Symbol!("Bar")>`).
pub fn is_blanket_trait(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    !CGP_CRATES.contains(&tcx.crate_name(def_id.krate).as_str())
        && has_blanket_impl(tcx, def_id)
        && blanket_depends_on_cgp(tcx, def_id, 0)
}

/// Whether `def_id` is a trait carrying at least one blanket impl.
pub fn has_blanket_impl(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    tcx.is_trait(def_id) && !tcx.trait_impls_of(def_id).blanket_impls().is_empty()
}

/// How far the CGP-evidence search follows one trait's blanket bounds into another's. A
/// composed chain (`Describe` → `HasName` → `HasField`) is a few links at most, and the bound is
/// also what stops a pair of traits that depend on each other from looping.
pub const MAX_BLANKET_TRAIT_DEPTH: u32 = 4;

/// Whether a blanket-impl trait's blanket actually depends on CGP — the positive evidence a trait
/// needs before it is read as a blanket trait. A bound qualifies when its trait comes
/// from one of [`CGP_CRATES`] (`HasField` above all), is a CGP consumer trait (recognized
/// structurally in any crate), or is itself such a blanket trait, followed to
/// [`MAX_BLANKET_TRAIT_DEPTH`].
pub fn blanket_depends_on_cgp(tcx: TyCtxt<'_>, def_id: DefId, depth: u32) -> bool {
    if depth >= MAX_BLANKET_TRAIT_DEPTH {
        return false;
    }
    tcx.trait_impls_of(def_id)
        .blanket_impls()
        .iter()
        .any(|&blanket| {
            tcx.clauses_of(blanket)
                .clauses
                .iter()
                .filter_map(|(clause, _)| clause.as_trait_clause())
                .any(|bound| {
                    let bound_did = bound.def_id();
                    is_cgp_crate_trait(tcx, bound_did)
                        || is_consumer_trait(tcx, bound_did)
                        || (has_blanket_impl(tcx, bound_did)
                            && blanket_depends_on_cgp(tcx, bound_did, depth + 1))
                })
        })
}
