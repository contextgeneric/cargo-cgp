//! Reading an impl's `where` clauses in the order the program wrote them.

use rustc_middle::ty::{self, TyCtxt};
use rustc_span::Span;

/// The order to read an impl's `clauses` in, as indices into the slice, so the walk descends an
/// impl's dependencies, and reports their causes, in the order the program wrote them.
///
/// The compiler stores the clauses in written order but then moves each associated-type constraint
/// (`Value = u64` in `HasField<Symbol!("f"), Value = u64>`) to the front, swapping it with the
/// clause there, which scrambles the trait clauses behind it. The constraints keep their written
/// order, so a trait clause that carries one takes its constraint's place, and the clauses without
/// one follow in the compiler's order. An impl with no constraint keeps its written order exactly,
/// and so does one whose every trait clause carries a constraint, as the bounds a getter or an
/// `#[implicit]` argument generates do. Source positions cannot restore the order: those generated
/// bounds all carry the macro's call site, and a macro mixes that site with the user's own tokens.
pub fn written_order<'tcx>(tcx: TyCtxt<'tcx>, clauses: &[(ty::Clause<'tcx>, Span)]) -> Vec<usize> {
    let constrained: Vec<Option<ty::TraitRef<'tcx>>> = clauses
        .iter()
        .map(|(clause, _)| {
            clause
                .as_projection_clause()
                .map(|projection| projection.skip_binder().projection_term.trait_ref(tcx))
        })
        .collect();
    let rank = |index: usize| {
        clauses[index]
            .0
            .as_trait_clause()
            .and_then(|trait_clause| {
                let trait_ref = trait_clause.skip_binder().trait_ref;
                constrained
                    .iter()
                    .position(|constraint| *constraint == Some(trait_ref))
            })
            .unwrap_or(index)
    };
    let mut order: Vec<usize> = (0..clauses.len()).collect();
    order.sort_by_key(|&index| (rank(index), index));
    order
}
