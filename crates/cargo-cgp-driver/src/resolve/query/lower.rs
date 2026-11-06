//! The small syntactic type lowering the call-site recovery stands on.
//!
//! This is deliberately not the compiler's HIR lowering: it runs inside the emitter, where only
//! already-cached queries may be forced, so it reads `type_of` for a named item (cached: typeck
//! resolved this very type to produce the diagnostic) and composes the rest by hand, declining
//! anything beyond it rather than guessing.

use rustc_hir::def::{CtorKind, DefKind, Res};
use rustc_hir::{self as hir, Expr, ExprKind, QPath};
use rustc_middle::ty::{self, Ty, TyCtxt, TypeVisitableExt as _};
use rustc_span::def_id::DefId;

/// Lower a *written* type to its `ty::Ty`, syntactically: a path to an ADT or alias (with written
/// arguments, a literal const argument among them, defaulted parameters filled in, lifetimes
/// erased), a primitive, a tuple, a reference, or a slice. `None` for any shape beyond it,
/// declining the anchor rather than guessing.
pub fn lower_hir_ty<'tcx>(tcx: TyCtxt<'tcx>, hir_ty: &hir::Ty<'tcx>) -> Option<Ty<'tcx>> {
    match hir_ty.kind {
        hir::TyKind::Path(QPath::Resolved(None, path)) => {
            let written = written_args(tcx, path)?;
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
        hir::TyKind::Ref(_, ty, mutbl) => Some(Ty::new_ref(
            tcx,
            tcx.lifetimes.re_erased,
            lower_hir_ty(tcx, ty)?,
            mutbl,
        )),
        hir::TyKind::Slice(ty) => Some(Ty::new_slice(tcx, lower_hir_ty(tcx, ty)?)),
        _ => None,
    }
}

/// One written generic argument of a path: a lowered type, or a const argument, which is lowered
/// once the parameter it fills, and so its type, is known.
pub enum WrittenArg<'a, 'tcx> {
    /// A lowered type.
    Type(Ty<'tcx>),
    /// A const argument, not yet lowered.
    Const(&'a hir::ConstArg<'tcx>),
}

/// The written generic arguments of a path's last segment, each type lowered; lifetimes are
/// skipped (they are re-supplied erased) and an inferred argument declines. An argument-less path
/// yields the empty list.
pub fn written_args<'a, 'tcx>(
    tcx: TyCtxt<'tcx>,
    path: &'a hir::Path<'tcx>,
) -> Option<Vec<WrittenArg<'a, 'tcx>>> {
    let Some(args) = path.segments.last()?.args else {
        return Some(Vec::new());
    };
    args.args
        .iter()
        .filter(|arg| !matches!(arg, hir::GenericArg::Lifetime(_)))
        .map(|arg| match arg {
            hir::GenericArg::Type(ty) => {
                Some(WrittenArg::Type(lower_hir_ty(tcx, ty.as_unambig_ty())?))
            }
            hir::GenericArg::Const(ct) => Some(WrittenArg::Const(ct.as_unambig_ct())),
            _ => None,
        })
        .collect()
}

/// Instantiate an ADT or type alias with its written arguments: lifetimes are erased, a const
/// parameter takes its written literal (the `3` and `'a'` of `Symbol<3, Chars<'a', …>>`, what
/// `Symbol!` expands to), missing trailing type parameters take their declared defaults, and any
/// other const argument or an arity mismatch declines.
pub fn instantiate_written<'tcx>(
    tcx: TyCtxt<'tcx>,
    did: DefId,
    written: Vec<WrittenArg<'_, 'tcx>>,
) -> Option<Ty<'tcx>> {
    if tcx.generics_of(did).parent.is_some() {
        return None;
    }
    let mut written = written.into_iter();
    let mut lowered = true;
    let args = ty::GenericArgs::for_item(tcx, did, |param, args_so_far| match param.kind {
        ty::GenericParamDefKind::Lifetime => tcx.lifetimes.re_erased.into(),
        ty::GenericParamDefKind::Type { .. } => match written.next() {
            Some(WrittenArg::Type(ty)) => ty.into(),
            None if let Some(default) = param.default_value(tcx) => {
                default.instantiate(tcx, args_so_far).skip_norm_wip()
            }
            _ => {
                lowered = false;
                tcx.types.unit.into()
            }
        },
        ty::GenericParamDefKind::Const { .. } => {
            let ty = tcx
                .type_of(param.def_id)
                .instantiate(tcx, args_so_far)
                .skip_norm_wip();
            match written.next() {
                Some(WrittenArg::Const(arg)) if let Some(ct) = lower_const_lit(tcx, arg, ty) => {
                    ct.into()
                }
                // A stand-in the decline below discards. An error constant would register a delayed
                // bug with the diagnostic context the emitter is inside.
                _ => {
                    lowered = false;
                    tcx.consts.unit.into()
                }
            }
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

/// A literal const argument of type `ty` (`3`, `'a'`, `-1`), written directly or as an anonymous
/// constant whose body is the literal, as the compiler's own type lowering reads it. The
/// `lit_to_const` query builds the value without evaluating anything, so it reports nothing.
fn lower_const_lit<'tcx>(
    tcx: TyCtxt<'tcx>,
    arg: &hir::ConstArg<'tcx>,
    ty: Ty<'tcx>,
) -> Option<ty::Const<'tcx>> {
    let (lit, neg) = match arg.kind {
        hir::ConstArgKind::Literal { lit, negated } => (lit, negated),
        hir::ConstArgKind::Anon(anon) => {
            let mut expr = tcx.hir_body(anon.body).value;
            if let ExprKind::Block(block, _) = expr.kind
                && block.stmts.is_empty()
                && let Some(inner) = block.expr
            {
                expr = inner;
            }
            match expr.kind {
                ExprKind::Lit(lit) => (lit.node, false),
                ExprKind::Unary(hir::UnOp::Neg, inner) => match inner.kind {
                    ExprKind::Lit(lit) => (lit.node, true),
                    _ => return None,
                },
                _ => return None,
            }
        }
        _ => return None,
    };
    let value = tcx.lit_to_const(ty::LitToConstInput {
        lit,
        ty: Some(ty),
        neg,
    })?;
    Some(ty::Const::new_value(tcx, value.valtree, value.ty))
}

/// The declared return type of a call to a non-generic fn named by path, or to a tuple-struct or
/// enum-variant constructor (`Shape::Circle(…)`), the expression shapes whose type a
/// (collection-cached) signature supplies without typeck. `None` for a generic callee (inference
/// would have decided its instantiation), a type-relative path (`Vec::new`, whose resolution lives
/// only in typeck results), or an output that still carries a late-bound region (relating it would
/// leak an escaping bound var).
pub fn call_output_ty<'tcx>(tcx: TyCtxt<'tcx>, callee: &Expr<'tcx>) -> Option<Ty<'tcx>> {
    let ExprKind::Path(QPath::Resolved(None, path)) = callee.kind else {
        return None;
    };
    let Res::Def(DefKind::Fn | DefKind::AssocFn | DefKind::Ctor(_, CtorKind::Fn), did) = path.res
    else {
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
pub fn prim_ty<'tcx>(tcx: TyCtxt<'tcx>, prim: hir::PrimTy) -> Ty<'tcx> {
    match prim {
        hir::PrimTy::Bool => tcx.types.bool,
        hir::PrimTy::Char => tcx.types.char,
        hir::PrimTy::Str => tcx.types.str_,
        hir::PrimTy::Int(int) => Ty::new_int(tcx, int),
        hir::PrimTy::Uint(uint) => Ty::new_uint(tcx, uint),
        hir::PrimTy::Float(float) => Ty::new_float(tcx, float),
    }
}

/// The identity type of an item (`type_of`), for the receivers whose type is an item's own, a
/// unit struct's, a const's, or a static's.
pub fn item_ty<'tcx>(tcx: TyCtxt<'tcx>, did: DefId) -> Option<Ty<'tcx>> {
    Some(tcx.type_of(did).instantiate_identity().skip_norm_wip())
}

/// Strip the reference layers off a written type (`&App` → `App`), since the receiver's context
/// is the ADT beneath them.
pub fn peel_hir_refs<'tcx>(mut ty: &'tcx hir::Ty<'tcx>) -> &'tcx hir::Ty<'tcx> {
    while let hir::TyKind::Ref(_, inner, _) = ty.kind {
        ty = inner;
    }
    ty
}
