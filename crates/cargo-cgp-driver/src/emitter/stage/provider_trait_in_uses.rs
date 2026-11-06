//! The stage for a provider trait named in `#[uses(…)]`.

use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::DiagInner;
use rustc_errors::codes::E0107;
use rustc_middle::ty::TyCtxt;

use crate::detect;
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};

/// Stage 3: reword the `E0107` that `#[uses(AreaCalculator)]` produces, where a dependency names
/// the consumer trait. rustc reports the provider trait one argument short and suggests the
/// generated `__Context__`, or a generic component's parameter; the reshape names the consumer
/// trait to declare and keeps the caret on the trait name.
pub fn provider_trait_in_uses(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if diag.code != Some(E0107) {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
        return Verdict::Continue;
    };
    match detect::detect_provider_trait_in_uses(tcx, primary_span) {
        Some(misuse) => Verdict::Reshape(Reshape::new(
            ClassPlan::provider_trait_in_uses(&misuse),
            SpanPolicy::Primary,
        )),
        None => Verdict::Continue,
    }
}
