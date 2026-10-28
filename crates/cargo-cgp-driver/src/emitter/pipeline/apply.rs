//! Writing a reshape's wording and span into the diagnostic.

use cargo_cgp_error_processing::ClassPlan;
use rustc_errors::{DiagInner, MultiSpan, Sublevel};

use crate::emitter::edit::{replace_header, subdiag, two_definitions_span};
use crate::emitter::pipeline::SpanPolicy;

/// Write `plan` into `diag` over the spans `span` keeps: the header replaces rustc's message, and
/// the plan's notes and then its helps replace rustc's notes and suggestions, so each fix follows
/// the explanation it repairs.
pub fn apply_reshape(diag: &mut DiagInner, plan: ClassPlan, span: SpanPolicy) {
    let primary = diag.span.primary_span();
    replace_header(diag, plan.header);
    match span {
        SpanPolicy::Keep => {}
        SpanPolicy::Primary => {
            if let Some(primary) = primary {
                diag.span = MultiSpan::from_span(primary);
            }
        }
        SpanPolicy::PrimaryCallSite => {
            if let Some(primary) = primary {
                diag.span = MultiSpan::from_span(primary.source_callsite());
            }
        }
        SpanPolicy::TwoDefinitions { first, second } => {
            diag.span = two_definitions_span(&diag.span, first, second);
        }
    }
    let notes = plan
        .notes
        .into_iter()
        .map(|note| subdiag(Sublevel::Note, note));
    let helps = plan
        .helps
        .into_iter()
        .map(|help| subdiag(Sublevel::Help, help));
    diag.children.extend(notes.chain(helps));
}
