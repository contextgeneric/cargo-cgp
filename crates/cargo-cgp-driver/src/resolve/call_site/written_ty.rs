//! Reading the type an argument expression writes syntactically.

use rustc_hir::def::{CtorKind, CtorOf, DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, QPath};
use rustc_infer::infer::InferCtxt;
use rustc_middle::ty::{Ty, TyCtxt};
use rustc_span::DUMMY_SP;

use crate::resolve::call_site::{call_output_ty, instantiate_written, item_ty, written_type_args};

/// The type an argument expression *writes*, syntactically — the call-side information the
/// signature unification consumes. Covered shapes: a unit-struct or unit-variant value path with
/// its written arguments (`PhantomData::<Program>`, `GetMethod`), a non-generic const, a struct
/// literal, a reference, a tuple (its *structure* recovered even when some elements are not
/// written), a literal whose type is definite (`"…"`, suffixed numerics, `true`, `'c'`), and a call
/// to a non-generic fn (its declared return type). `None` for anything whose type only inference
/// could know — an unsuffixed literal, a variable, a generic constructor like `Vec::new()` —
/// leaving the corresponding parameter unknown rather than guessed.
pub(crate) fn expr_written_ty<'tcx>(
    infcx: &InferCtxt<'tcx>,
    expr: &Expr<'tcx>,
) -> Option<Ty<'tcx>> {
    let tcx = infcx.tcx;
    match expr.kind {
        ExprKind::Path(QPath::Resolved(None, path)) => match path.res {
            // A unit-struct/unit-variant value: its type is the ADT, with whatever arguments the
            // path writes (defaults filled in; a generic ADT with none written stays unknown).
            Res::Def(DefKind::Ctor(ctor_of, CtorKind::Const), ctor_did) => {
                let adt_did = match ctor_of {
                    CtorOf::Struct => tcx.parent(ctor_did),
                    CtorOf::Variant => tcx.parent(tcx.parent(ctor_did)),
                };
                instantiate_written(tcx, adt_did, written_type_args(tcx, path)?)
            }
            Res::Def(DefKind::Const { .. }, did) if tcx.generics_of(did).is_empty() => {
                item_ty(tcx, did)
            }
            _ => None,
        },
        ExprKind::Struct(qpath, ..) => {
            let QPath::Resolved(None, path) = qpath else {
                return None;
            };
            let Res::Def(DefKind::Struct, did) = path.res else {
                return None;
            };
            instantiate_written(tcx, did, written_type_args(tcx, path)?)
        }
        ExprKind::AddrOf(_, mutbl, inner) => Some(Ty::new_ref(
            tcx,
            tcx.lifetimes.re_erased,
            expr_written_ty(infcx, inner)?,
            mutbl,
        )),
        // A tuple literal writes its *shape*, whether or not every element's type is written. An
        // element the call does not type becomes a fresh inference variable (folded into a
        // placeholder with the rest of the seed by
        // [`unknowns_to_placeholders`](crate::resolve::walk::unknowns_to_placeholders)), so the
        // tuple arity and its written elements are recovered even beside an unknown one. This
        // matters because providers destructure their input on the tuple shape — `HandleIf`'s
        // `(InputCond, InputBranch)`, `HandleCompare`'s `(InputA, InputB)` — so collapsing the
        // whole tuple to one flat unknown (as returning `None` would) leaves such a provider's
        // impl unmatched and hides a cause sitting inside a *known* branch (a field read by the
        // condition, say). The recovered structure is real call-side information, not a guess:
        // the leaves it cannot type stay unknown and are never reported.
        ExprKind::Tup(elems) => {
            let tys: Vec<Ty<'tcx>> = elems
                .iter()
                .map(|elem| {
                    expr_written_ty(infcx, elem).unwrap_or_else(|| infcx.next_ty_var(DUMMY_SP))
                })
                .collect();
            Some(Ty::new_tup(tcx, &tys))
        }
        ExprKind::Lit(lit) => lit_ty(tcx, &lit),
        ExprKind::Call(callee, _) => call_output_ty(tcx, callee),
        _ => None,
    }
}

/// The definite type of a literal — `None` for the suffixless numerics whose type only inference
/// decides.
fn lit_ty<'tcx>(tcx: TyCtxt<'tcx>, lit: &hir::Lit) -> Option<Ty<'tcx>> {
    use rustc_ast::{LitFloatType, LitIntType, LitKind};
    match lit.node {
        LitKind::Str(..) => Some(Ty::new_imm_ref(
            tcx,
            tcx.lifetimes.re_erased,
            tcx.types.str_,
        )),
        LitKind::Bool(_) => Some(tcx.types.bool),
        LitKind::Char(_) => Some(tcx.types.char),
        LitKind::Byte(_) => Some(tcx.types.u8),
        LitKind::Int(_, LitIntType::Signed(int)) => Some(Ty::new_int(tcx, int)),
        LitKind::Int(_, LitIntType::Unsigned(uint)) => Some(Ty::new_uint(tcx, uint)),
        LitKind::Float(_, LitFloatType::Suffixed(float)) => Some(Ty::new_float(tcx, float)),
        _ => None,
    }
}
