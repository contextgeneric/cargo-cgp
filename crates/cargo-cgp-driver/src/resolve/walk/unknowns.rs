//! Carrying an unknown call input across inference-context boundaries.
//!
//! A call-site parameter the code never types stays an inference variable, which cannot leave
//! its `InferCtxt`. These folders turn such unknowns into rigid placeholders the walk can carry
//! (and back, when a stalled projection must be reduced with the unknown treated as deferrable).

use rustc_infer::infer::{InferCtxt, TyCtxtInferExt};
use rustc_middle::ty::{
    self, Ty, TyCtxt, TypeFoldable, TypeFolder, TypeSuperFoldable, TypeVisitableExt, TypingMode,
    Unnormalized,
};
use rustc_span::DUMMY_SP;
use rustc_trait_selection::traits::{ObligationCause, ObligationCtxt};

/// Replace every unresolved inference variable in `value` with a rigid placeholder, so it can
/// cross inference-context boundaries (a placeholder is a rigid type constant, not tied to an
/// `InferCtxt`). Distinct variables get distinct placeholders and repeated occurrences of one
/// variable the same one (keyed by the variable's index, the integer and float spaces offset
/// apart), preserving whatever type equalities unification established. Used both to seed the
/// [call-site anchor](crate::resolve::call_site)'s unknown call arguments and to keep a
/// later-pipeline-stage clause walkable in
/// [`impl_where_obligations`](super::impl_where_obligations).
pub(crate) fn unknowns_to_placeholders<'tcx, T>(tcx: TyCtxt<'tcx>, value: T) -> T
where
    T: TypeFoldable<TyCtxt<'tcx>>,
{
    struct Folder<'tcx> {
        tcx: TyCtxt<'tcx>,
    }
    impl<'tcx> TypeFolder<TyCtxt<'tcx>> for Folder<'tcx> {
        fn cx(&self) -> TyCtxt<'tcx> {
            self.tcx
        }
        fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
            let var = match *ty.kind() {
                ty::Infer(ty::TyVar(vid)) => vid.as_u32(),
                ty::Infer(ty::IntVar(vid)) => (1 << 20) + vid.as_u32(),
                ty::Infer(ty::FloatVar(vid)) => (1 << 21) + vid.as_u32(),
                ty::Infer(_) => 1 << 22,
                _ if !ty.has_infer() => return ty,
                _ => return ty.super_fold_with(self),
            };
            Ty::new_placeholder(
                self.tcx,
                ty::PlaceholderType::new_anon(ty::UniverseIndex::ROOT, ty::BoundVar::from_u32(var)),
            )
        }
    }
    value.fold_with(&mut Folder { tcx })
}

/// Replace any **fixed** associated-type projection in `clause` that stalls on an unknown-input
/// placeholder with the concrete type it reduces to. See the call site in
/// [`impl_where_obligations`](super::impl_where_obligations) for why this matters: a later
/// pipeline stage's input is an earlier stage's `::Output`, and against a rigid placeholder input
/// the earlier provider's `where` clause is *false* (not merely unproven), so the projection never
/// reduces even though the provider's `type Output` is independent of the input. A no-op unless
/// the clause carries a placeholder.
pub(crate) fn resolve_fixed_projections<'tcx>(
    tcx: TyCtxt<'tcx>,
    clause: ty::Clause<'tcx>,
) -> ty::Clause<'tcx> {
    if !clause.has_placeholders() {
        return clause;
    }
    clause.fold_with(&mut ProjectionResolver { tcx })
}

/// Folds a value, replacing each associated-type projection that both carries an unknown-input
/// placeholder and reduces to a concrete type (via [`try_project_fixed`]) with that concrete type.
struct ProjectionResolver<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> TypeFolder<TyCtxt<'tcx>> for ProjectionResolver<'tcx> {
    fn cx(&self) -> TyCtxt<'tcx> {
        self.tcx
    }
    fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
        // Only a projection that mentions an unknown can be one that stalled on the unknown input;
        // a placeholder-free subtree is left untouched (and its recursion pruned).
        if !ty.has_placeholders() {
            return ty;
        }
        if let ty::Alias(_, alias) = ty.kind()
            && matches!(alias.kind, ty::AliasTyKind::Projection { .. })
            && let Some(concrete) = try_project_fixed(self.tcx, ty)
        {
            return concrete;
        }
        ty.super_fold_with(self)
    }
}

/// Try to reduce one associated-type projection `alias_ty` to a concrete type, treating the
/// unknown-input placeholders inside it as *deferrable* inference variables rather than rigid
/// falsifiers. Returns the reduced type only when it is fully concrete — no inference variable or
/// placeholder left (so it does not depend on the unknown input) and no longer an alias (the
/// projection genuinely reduced). `None` otherwise, leaving the projection as it was.
///
/// This is the one extra trait-solver interaction the walk performs here, and it is the safe kind:
/// a fresh `InferCtxt` normalizing a concrete goal, forcing only the cached queries type-checking
/// already ran (see the panic hazards in `docs/implementation/rustc-diagnostic-internals.md`).
fn try_project_fixed<'tcx>(tcx: TyCtxt<'tcx>, alias_ty: Ty<'tcx>) -> Option<Ty<'tcx>> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let ocx = ObligationCtxt::new(&infcx);
    let goal = placeholders_to_infer(&infcx, alias_ty);
    let normalized = ocx.normalize(
        &ObligationCause::dummy(),
        ty::ParamEnv::empty(),
        Unnormalized::new_wip(goal),
    );
    let _ = ocx.try_evaluate_obligations();
    let normalized = tcx.erase_and_anonymize_regions(infcx.resolve_vars_if_possible(normalized));
    if normalized.has_non_region_infer()
        || normalized.has_placeholders()
        || matches!(normalized.kind(), ty::Alias(..))
    {
        return None;
    }
    Some(normalized)
}

/// Replace every unknown-input placeholder (a root-universe anonymous placeholder, as minted by
/// [`unknowns_to_placeholders`]) with a fresh inference variable, one per distinct placeholder so
/// equalities are preserved. A leaked higher-ranked binder's placeholder (a non-root universe) is
/// left alone. The inverse of [`unknowns_to_placeholders`], used to let the solver *defer* a bound
/// on the unknown rather than reject it while a fixed projection is reduced.
fn placeholders_to_infer<'tcx, T>(infcx: &InferCtxt<'tcx>, value: T) -> T
where
    T: TypeFoldable<TyCtxt<'tcx>>,
{
    struct Folder<'a, 'tcx> {
        infcx: &'a InferCtxt<'tcx>,
        vars: std::collections::HashMap<u32, Ty<'tcx>>,
    }
    impl<'tcx> TypeFolder<TyCtxt<'tcx>> for Folder<'_, 'tcx> {
        fn cx(&self) -> TyCtxt<'tcx> {
            self.infcx.tcx
        }
        fn fold_ty(&mut self, ty: Ty<'tcx>) -> Ty<'tcx> {
            if let ty::Placeholder(p) = *ty.kind()
                && p.universe == ty::UniverseIndex::ROOT
            {
                let key = p.bound.var.as_u32();
                return *self
                    .vars
                    .entry(key)
                    .or_insert_with(|| self.infcx.next_ty_var(DUMMY_SP));
            }
            if !ty.has_placeholders() {
                return ty;
            }
            ty.super_fold_with(self)
        }
    }
    value.fold_with(&mut Folder {
        infcx,
        vars: std::collections::HashMap::new(),
    })
}
