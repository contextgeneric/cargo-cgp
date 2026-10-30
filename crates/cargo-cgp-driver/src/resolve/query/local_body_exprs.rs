//! The expressions of the bodies the crate being compiled writes.

use rustc_hir::Expr;
use rustc_hir::intravisit::{self, Visitor};
use rustc_middle::ty::TyCtxt;

/// Every expression in the local crate's bodies: the bodies in the order `hir_body_owners` lists
/// them, and in each body every expression before the expressions inside it. The caller keeps the
/// expressions it wants, such as the calls a diagnostic's spans are about.
pub fn local_body_exprs<'tcx>(tcx: TyCtxt<'tcx>) -> impl Iterator<Item = &'tcx Expr<'tcx>> {
    tcx.hir_body_owners()
        .flat_map(move |owner| body_exprs(tcx.hir_body_owned_by(owner).value))
}

/// Every expression in `root`, `root` included, each before the expressions inside it.
fn body_exprs<'tcx>(root: &'tcx Expr<'tcx>) -> Vec<&'tcx Expr<'tcx>> {
    struct Collector<'tcx> {
        exprs: Vec<&'tcx Expr<'tcx>>,
    }
    impl<'tcx> Visitor<'tcx> for Collector<'tcx> {
        fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
            self.exprs.push(expr);
            intravisit::walk_expr(self, expr);
        }
    }
    let mut collector = Collector { exprs: Vec::new() };
    collector.visit_expr(root);
    collector.exprs
}
