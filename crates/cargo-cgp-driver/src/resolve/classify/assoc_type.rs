//! Reading the type an owner actually supplies for a projected associated type.

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt, TypingMode, Unnormalized};
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

/// The concrete type `projection` resolves to — `String` for `<App as HasErrorType>::Error` on a
/// context that wired `UseType<String>` — or `None` when it does not reduce to one. This is the
/// mismatch leaf's *actual* side, the counterpart of the *expected* type read off the failing
/// projection, and it is the general form of the struct-field query
/// [`field_type`](super::field_type) performs for a `HasField` value.
///
/// It is read by asking the trait solver to normalize the alias, which is exactly how the owner's
/// choice is recorded: an abstract type wired to `UseType<T>` resolves through the blanket impl
/// `#[cgp_type]` generates, and a hand-written `impl HasErrorType for App { type Error = String; }`
/// resolves through that impl, so both forms are read the same way with no wiring-shape special
/// case. Only a **fully concrete** result is kept — an alias or inference variable left over means
/// the owner's choice is not pinned down, and reporting a half-resolved type would mislead.
///
/// The obligations normalization registers are deliberately *not* discharged: the question is what
/// the owner supplies, not whether supplying it is well-formed, and the answer is used only to
/// render one side of a message. An ambiguous normalization leaves an inference variable behind and
/// so is rejected by the concreteness check above, which is the guard that matters here.
///
/// The normalization runs in its own throwaway `InferCtxt`, and nothing crosses back out of it but
/// the owned rendered type, so it cannot leak a variable into the walk's contexts (the
/// cross-context contamination hazard in `docs/implementation/rustc-diagnostic-internals.md`).
pub(crate) fn projected_type<'tcx>(tcx: TyCtxt<'tcx>, alias: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let param_env = ty::ParamEnv::empty();
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);

    let normalized = ocx.normalize(
        &ObligationCause::dummy(),
        param_env,
        Unnormalized::new_wip(alias),
    );
    let normalized = tcx.erase_and_anonymize_regions(infcx.resolve_vars_if_possible(normalized));

    // A still-aliased or inference-laden result means the projection did not reduce, so there is no
    // actual type to report.
    if normalized.has_non_region_infer() || matches!(normalized.kind(), ty::Alias(..)) {
        return None;
    }
    Some(normalized)
}
