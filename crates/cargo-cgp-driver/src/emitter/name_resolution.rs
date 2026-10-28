//! Reshaping the name-resolution errors of a wiring entry or a `#[cgp_impl]` header.
//!
//! A wiring key or value that names nothing, and a `#[cgp_impl(Provider)]` that never declares its
//! provider, fail during name resolution with an `E0425` "cannot find type", before the compiler has
//! built the HIR. The emitter buffers those diagnostics as rustc wrote them. Once resolution is over
//! the driver's `after_expansion` callback runs [`reshape_name_resolution_errors`], which reads the
//! wiring entries back from the HIR ([`detect::plan_unresolved_names`]) and rewrites the buffered
//! diagnostics in place: a coded header with the fix, or the diagnostic dropped as a repeat or a
//! consequence.
//!
//! The buffer is read and written under the output's lock, and the planning, which runs compiler
//! queries, runs with the lock released, since a query that emits a diagnostic takes that lock.
//!
//! A wiring entry whose value does not resolve also leaves its context unable to use the component,
//! which a call later reports as an `E0275` overflow on that requirement. The pass records those
//! requirements in the output, and the emitter drops such an overflow as the entry's consequence
//! ([`is_unresolved_wiring_overflow`]).

use cargo_cgp_error_processing::{
    ClassPlan, UnresolvedWiringName, is_missing_type_parameter_help_text,
};
use rustc_errors::codes::{E0275, E0425};
use rustc_errors::{DiagInner, DiagMessage, Style, Sublevel, Suggestions};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::detect::{self, NameAction};
use crate::emitter::buffer::BufEntry;
use crate::emitter::buffer_pass::buffer_pass;
use crate::emitter::edit::{main_message_text, retain_suggestions, subdiag};
use crate::emitter::output::record_unprovable_requirements;
use crate::emitter::pipeline::{SpanPolicy, apply_reshape};

/// Whether `diag` is an `E0275` overflow on one of the `unprovable` requirements a wiring entry
/// with an unresolved value left (`overflow evaluating the requirement `Person: CanGreet``), the
/// consequence of the entry's own error. A recorded requirement stops before the component's
/// parameters, so the overflow's may go on with a consumer trait's arguments (`<`) or the rest of a
/// check's (`>`, `,`).
pub fn is_unresolved_wiring_overflow(diag: &DiagInner, unprovable: &[String]) -> bool {
    if diag.code != Some(E0275) {
        return false;
    }
    let Some(requirement) = main_message_text(diag)
        .and_then(|text| text.strip_prefix("overflow evaluating the requirement `"))
        .and_then(|text| text.strip_suffix('`'))
    else {
        return false;
    };
    unprovable.iter().any(|unprovable| {
        requirement
            .strip_prefix(unprovable.as_str())
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(['<', '>', ',']))
    })
}

/// Reshape the buffered name-resolution `E0425`s that a wiring entry or a `#[cgp_impl]` header
/// produced, and record in the output the requirements the entries with an unresolved value leave
/// unprovable. Call once name resolution is over, while a `TyCtxt` is available. A panic in the
/// planning leaves every diagnostic as rustc wrote it.
pub fn reshape_name_resolution_errors(tcx: TyCtxt<'_>) {
    buffer_pass(
        unresolved_names,
        |candidates| {
            let names: Vec<(Span, String)> = candidates
                .iter()
                .map(|(_, span, name)| (*span, name.clone()))
                .collect();
            let plan = detect::plan_unresolved_names(tcx, &names);
            record_unprovable_requirements(plan.overflow_requirements);
            plan.actions
        },
        |buffer, candidates, actions| {
            let mut dropped = Vec::new();
            for ((index, ..), action) in candidates.iter().zip(actions) {
                match action {
                    // The candidates are unchanged, so the diagnostic's primary span is still the
                    // name's span.
                    Some(NameAction::Reword(name)) => {
                        let Some(BufEntry::Plain(diag)) = buffer.get_mut(*index) else {
                            continue;
                        };
                        apply_reshape(
                            diag,
                            ClassPlan::unresolved_wiring_name(&name),
                            SpanPolicy::Primary,
                        );
                    }
                    Some(NameAction::RewordHeader(name)) => {
                        let Some(BufEntry::Plain(diag)) = buffer.get_mut(*index) else {
                            continue;
                        };
                        reword_header_keeping_advice(diag, &name);
                    }
                    Some(NameAction::Drop) => dropped.push(*index),
                    None => {}
                }
            }
            for index in dropped.into_iter().rev() {
                buffer.remove(index);
            }
        },
    );
}

/// Replace the header of `diag` with the coded form of `name`, keeping the compiler's notes and
/// suggestions except the one to add a type parameter, which no wiring entry can take. The coded
/// help is added only when nothing else is left that says how to fix the name.
fn reword_header_keeping_advice(diag: &mut DiagInner, name: &UnresolvedWiringName) {
    let plan = ClassPlan::unresolved_wiring_name(name);
    diag.messages = vec![(DiagMessage::Str(plan.header.into()), Style::NoStyle)];
    retain_suggestions(
        diag,
        |suggestion| !matches!(&suggestion.msg, DiagMessage::Str(text) if is_missing_type_parameter_help_text(text)),
    );
    let has_suggestion = match &diag.suggestions {
        Suggestions::Enabled(suggestions) => !suggestions.is_empty(),
        Suggestions::Sealed(suggestions) => !suggestions.is_empty(),
        Suggestions::Disabled => false,
    };
    if !has_suggestion && diag.children.is_empty() {
        diag.children.extend(
            plan.helps
                .into_iter()
                .map(|help| subdiag(Sublevel::Help, help)),
        );
    }
}

/// Each buffered `E0425` "cannot find type `name` in this scope", as its buffer index, primary
/// span, and name, in buffer order.
fn unresolved_names(buffer: &[BufEntry]) -> Vec<(usize, Span, String)> {
    buffer
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let BufEntry::Plain(diag) = entry else {
                return None;
            };
            if diag.code != Some(E0425) {
                return None;
            }
            let name = main_message_text(diag)?
                .strip_prefix("cannot find type `")?
                .strip_suffix("` in this scope")?;
            Some((index, diag.span.primary_span()?, name.to_owned()))
        })
        .collect()
}
