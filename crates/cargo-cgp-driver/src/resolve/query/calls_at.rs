//! The calls whose called name sits at a diagnostic's span.

use rustc_hir::{self as hir, Expr, ExprKind, HirId, PathSegment, QPath};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use super::local_body_exprs;

/// A call whose method or function segment overlaps a span.
pub enum CallAt<'tcx> {
    /// A method call, `receiver.method(args)`.
    Method {
        /// The call expression.
        hir_id: HirId,
        /// The method's segment.
        segment: &'tcx PathSegment<'tcx>,
        /// The receiver expression.
        receiver: &'tcx Expr<'tcx>,
        /// The arguments after the receiver.
        args: &'tcx [Expr<'tcx>],
    },
    /// A call through a type-relative path, `Type::function(args)`.
    TypeRelative {
        /// The call expression.
        hir_id: HirId,
        /// The qualifying type as written.
        qself: &'tcx hir::Ty<'tcx>,
        /// The function's segment.
        segment: &'tcx PathSegment<'tcx>,
        /// The arguments.
        args: &'tcx [Expr<'tcx>],
    },
}

/// Every call in the local crate's bodies whose method or function segment overlaps `span`, in the
/// order of [`local_body_exprs`](fn@local_body_exprs). Matching the segment's own span keeps out a
/// call nested beside the failing one in the same expression (a sibling argument of one `format!`,
/// say). A caller that needs one call keeps the kinds it accepts and takes the last.
pub fn calls_at<'tcx>(tcx: TyCtxt<'tcx>, span: Span) -> impl Iterator<Item = CallAt<'tcx>> {
    local_body_exprs(tcx).filter_map(move |expr| match expr.kind {
        ExprKind::MethodCall(segment, receiver, args, _) if segment.ident.span.overlaps(span) => {
            Some(CallAt::Method {
                hir_id: expr.hir_id,
                segment,
                receiver,
                args,
            })
        }
        ExprKind::Call(callee, args) => match callee.kind {
            ExprKind::Path(QPath::TypeRelative(qself, segment))
                if segment.ident.span.overlaps(span) =>
            {
                Some(CallAt::TypeRelative {
                    hir_id: expr.hir_id,
                    qself,
                    segment,
                    args,
                })
            }
            _ => None,
        },
        _ => None,
    })
}
