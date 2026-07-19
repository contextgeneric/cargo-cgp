//! The small syntactic type lowering the call-site recovery stands on.
//!
//! This is deliberately not the compiler's HIR lowering: it runs inside the emitter, where only
//! already-cached queries may be forced, so it reads `type_of` for a named item (cached — typeck
//! resolved this very type to produce the diagnostic) and composes the rest by hand, declining
//! anything beyond it rather than guessing.

use rustc_hir::def::{DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, QPath};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;

/// Lower a *written* type to its `ty::Ty`, syntactically — a path to an ADT or alias (with
/// written arguments, defaulted parameters filled in, lifetimes erased), a primitive, a tuple, a
/// reference, or a slice. `None` for any shape beyond it, declining the anchor rather than
/// guessing.
pub(crate) fn lower_hir_ty<'tcx>(tcx: TyCtxt<'tcx>, hir_ty: &hir::Ty<'tcx>) -> Option<Ty<'tcx>> {
    match hir_ty.kind {
        hir::TyKind::Path(QPath::Resolved(None, path)) => {
            let written = written_type_args(tcx, path)?;
            match path.res {
                Res::PrimTy(prim) if written.is_empty() => Some(prim_ty(tcx, prim)),
                Res::Def(
                    DefKind::Struct | DefKind::Enum | DefKind::Union | DefKind::TyAlias,
                    did,
                ) => instantiate_written(tcx, did, written),
                _ => None,
            }
        }
        hir::TyKind::Tup(tys) => {
            let elems: Vec<Ty<'tcx>> = tys
                .iter()
                .map(|ty| lower_hir_ty(tcx, ty))
                .collect::<Option<_>>()?;
            Some(Ty::new_tup(tcx, &elems))
        }
        hir::TyKind::Ref(_, mut_ty) => Some(Ty::new_ref(
            tcx,
            tcx.lifetimes.re_erased,
            lower_hir_ty(tcx, mut_ty.ty)?,
            mut_ty.mutbl,
        )),
        hir::TyKind::Slice(ty) => Some(Ty::new_slice(tcx, lower_hir_ty(tcx, ty)?)),
        _ => None,
    }
}

/// The written type arguments of a path's last segment, each lowered; lifetimes are skipped
/// (they are re-supplied erased) and a const or inferred argument declines. An argument-less
/// path yields the empty list.
pub(crate) fn written_type_args<'tcx>(
    tcx: TyCtxt<'tcx>,
    path: &hir::Path<'tcx>,
) -> Option<Vec<Ty<'tcx>>> {
    let Some(args) = path.segments.last()?.args else {
        return Some(Vec::new());
    };
    args.args
        .iter()
        .filter(|arg| !matches!(arg, hir::GenericArg::Lifetime(_)))
        .map(|arg| match arg {
            hir::GenericArg::Type(ty) => lower_hir_ty(tcx, ty.as_unambig_ty()),
            _ => None,
        })
        .collect()
}

/// Instantiate an ADT or type alias with its written type arguments: lifetimes are erased,
/// missing trailing parameters take their declared defaults, and a const parameter or an arity
/// mismatch declines.
pub(crate) fn instantiate_written<'tcx>(
    tcx: TyCtxt<'tcx>,
    did: DefId,
    written: Vec<Ty<'tcx>>,
) -> Option<Ty<'tcx>> {
    if tcx.generics_of(did).parent.is_some() {
        return None;
    }
    let mut written = written.into_iter();
    let mut lowered = true;
    let args = ty::GenericArgs::for_item(tcx, did, |param, args_so_far| match param.kind {
        ty::GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
        ty::GenericParamDefKind::Type { .. } => {
            if let Some(ty) = written.next() {
                ty.into()
            } else if let Some(default) = param.default_value(tcx) {
                default.instantiate(tcx, args_so_far).skip_norm_wip()
            } else {
                lowered = false;
                tcx.types.unit.into()
            }
        }
        ty::GenericParamDefKind::Const { .. } => {
            lowered = false;
            tcx.types.unit.into()
        }
    });
    if !lowered || written.next().is_some() {
        return None;
    }
    if tcx.def_kind(did) == DefKind::TyAlias {
        return Some(tcx.type_of(did).instantiate(tcx, args).skip_norm_wip());
    }
    Some(Ty::new_adt(tcx, tcx.adt_def(did), args))
}

/// The declared return type of a call to a non-generic fn named by path — the one expression
/// shape whose type a (collection-cached) signature supplies without typeck. `None` for a generic
/// callee (its instantiation is exactly what inference would have decided), a type-relative path
/// (`Vec::new`, whose resolution lives only in typeck results), or an output that still carries a
/// late-bound region (relating it would leak an escaping bound var).
pub(crate) fn call_output_ty<'tcx>(tcx: TyCtxt<'tcx>, callee: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    let ExprKind::Path(QPath::Resolved(None, path)) = callee.kind else {
        return None;
    };
    let Res::Def(DefKind::Fn | DefKind::AssocFn, did) = path.res else {
        return None;
    };
    if !tcx.generics_of(did).is_empty() {
        return None;
    }
    let output = tcx
        .fn_sig(did)
        .instantiate_identity()
        .skip_norm_wip()
        .skip_binder()
        .output();
    (!output.has_escaping_bound_vars()).then_some(output)
}

/// The written primitive as its `ty::Ty`.
pub(crate) fn prim_ty<'tcx>(tcx: TyCtxt<'tcx>, prim: hir::PrimTy) -> Ty<'tcx> {
    match prim {
        hir::PrimTy::Bool => tcx.types.bool,
        hir::PrimTy::Char => tcx.types.char,
        hir::PrimTy::Str => tcx.types.str_,
        hir::PrimTy::Int(int) => Ty::new_int(tcx, int),
        hir::PrimTy::Uint(uint) => Ty::new_uint(tcx, uint),
        hir::PrimTy::Float(float) => Ty::new_float(tcx, float),
    }
}

/// The identity type of an item (`type_of`), for the receivers whose type is an item's own — a
/// unit struct's, a const's, or a static's.
pub(crate) fn item_ty<'tcx>(tcx: TyCtxt<'tcx>, did: DefId) -> Option<Ty<'tcx>> {
    Some(tcx.type_of(did).instantiate_identity().skip_norm_wip())
}

/// Strip the reference layers off a written type (`&App` → `App`), since the receiver's context
/// is the ADT beneath them.
pub(crate) fn peel_hir_refs<'tcx>(mut ty: &'tcx hir::Ty<'tcx>) -> &'tcx hir::Ty<'tcx> {
    while let hir::TyKind::Ref(_, mut_ty) = ty.kind {
        ty = mut_ty.ty;
    }
    ty
}
