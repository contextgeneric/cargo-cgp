//! The stage for a consumer trait written as a wiring key.

use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::DiagInner;
use rustc_errors::codes::E0782;
use rustc_middle::ty::TyCtxt;

use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};
use crate::resolve;

/// Stage 5: reword the `E0782` a consumer trait as a wiring key produces. rustc reports a trait
/// where a type belongs and suggests `dyn`; the reshape names the component the key should be.
pub fn consumer_trait_as_key(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if diag.code != Some(E0782) {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
        return Verdict::Continue;
    };
    match resolve::detect_consumer_trait_as_key(tcx, primary_span) {
        Some(name) => Verdict::Reshape(Reshape::new(
            ClassPlan::unresolved_wiring_name(&name),
            SpanPolicy::Primary,
        )),
        None => Verdict::Continue,
    }
}
