//! The stage for a provider header whose derived component lacks the parameters its key takes.

use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::DiagInner;
use rustc_errors::codes::E0107;
use rustc_middle::ty::TyCtxt;

use crate::detect;
use crate::emitter::pipeline::{Reshape, SpanPolicy, Verdict};

/// Reword the `E0107` a `#[cgp_impl]` or `#[cgp_provider]` header gives when its component's key
/// takes parameters, as `name: AreaCalculatorComponent<Shape>` declares, and the header derives the
/// key without them. rustc suggests adding the argument to the trait's name, which does not reach
/// the key; the reshape names the key with the header's arguments in the attribute. It is
/// confirmed structurally: the caret is a provider header's trait name, and that trait's component
/// is the derived name with parameters of its own.
pub fn derived_component_without_args(tcx: Option<TyCtxt<'_>>, diag: &DiagInner) -> Verdict {
    if diag.code != Some(E0107) {
        return Verdict::Continue;
    }
    let (Some(tcx), Some(primary_span)) = (tcx, diag.span.primary_span()) else {
        return Verdict::Continue;
    };
    let Some(header) = detect::provider_header_at(tcx, primary_span) else {
        return Verdict::Continue;
    };
    let derived = header.derived();
    if tcx.item_name(header.marker).as_str() != derived
        || tcx.generics_of(header.marker).own_params.is_empty()
    {
        return Verdict::Continue;
    }
    match detect::derived_component_name(tcx, &header) {
        Some(name) => Verdict::Reshape(Reshape::new(
            ClassPlan::unresolved_wiring_name(&name),
            SpanPolicy::Primary,
        )),
        None => Verdict::Continue,
    }
}
