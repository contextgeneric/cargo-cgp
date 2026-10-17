//! Reshaping the name-resolution errors of a wiring entry or a `#[cgp_impl]` header.
//!
//! A wiring key or value that names nothing, and a `#[cgp_impl(Provider)]` that never declares its
//! provider, fail during name resolution with an `E0425` "cannot find type", before the compiler has
//! built the HIR. The emitter buffers those diagnostics as rustc wrote them. Once resolution is over
//! the driver's `after_expansion` callback runs [`reshape_name_resolution_errors`], which reads the
//! wiring entries back from the HIR ([`resolve::plan_unresolved_names`]) and rewrites the buffered
//! diagnostics in place: a coded header with the fix, or the diagnostic dropped as a repeat or a
//! consequence.
//!
//! The buffer is read and written under the output's lock, and the planning, which runs compiler
//! queries, runs with the lock released, since a query that emits a diagnostic takes that lock.
//!
//! A wiring entry whose value does not resolve also leaves its context unable to use the component,
//! which a call later reports as an `E0275` overflow on that requirement. The pass records those
//! requirements, and the emitter drops such an overflow as the entry's consequence
//! ([`is_unresolved_wiring_overflow`]).

use std::cell::RefCell;

use cargo_cgp_error_processing::{
    UnresolvedWiringName, is_missing_type_parameter_help_text, plan_unresolved_wiring_name,
    unresolved_wiring_name_help,
};
use rustc_errors::codes::{E0275, E0425};
use rustc_errors::{DiagInner, DiagMessage, MultiSpan, Style, Sublevel, Suggestions};
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;

use crate::emitter::buffer::BufEntry;
use crate::emitter::containment::contain;
use crate::emitter::edit::{main_message_text, replace_header, retain_suggestions, subdiag};
use crate::emitter::output::with_registered_buffer;
use crate::resolve::{self, NameAction};

thread_local! {
    /// The requirements the wiring entries with an unresolved value leave unprovable, recorded by
    /// [`reshape_name_resolution_errors`] on the thread that compiles the crate.
    static UNPROVABLE_REQUIREMENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Whether `diag` is an `E0275` overflow on a requirement a wiring entry with an unresolved value
/// left unprovable (`overflow evaluating the requirement `Person: CanGreet``), the consequence of
/// the entry's own error.
pub(crate) fn is_unresolved_wiring_overflow(diag: &DiagInner) -> bool {
    if diag.code != Some(E0275) {
        return false;
    }
    let Some(requirement) = main_message_text(diag)
        .and_then(|text| text.strip_prefix("overflow evaluating the requirement `"))
        .and_then(|text| text.strip_suffix('`'))
    else {
        return false;
    };
    UNPROVABLE_REQUIREMENTS
        .try_with(|recorded| {
            recorded.borrow().iter().any(|unprovable| {
                requirement
                    .strip_prefix(unprovable.as_str())
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('<'))
            })
        })
        .unwrap_or(false)
}

/// Reshape the buffered name-resolution `E0425`s that a wiring entry or a `#[cgp_impl]` header
/// produced. Call once name resolution is over, while a `TyCtxt` is available. A panic in the
/// planning leaves every diagnostic as rustc wrote it.
pub fn reshape_name_resolution_errors(tcx: TyCtxt<'_>) {
    let Some(candidates) = with_registered_buffer(|buffer| unresolved_names(buffer)) else {
        return;
    };
    if candidates.is_empty() {
        return;
    }
    let names: Vec<(Span, String)> = candidates
        .iter()
        .map(|(_, span, name)| (*span, name.clone()))
        .collect();
    let Ok(plan) = contain(|| resolve::plan_unresolved_names(tcx, &names)) else {
        return;
    };
    let _ = UNPROVABLE_REQUIREMENTS
        .try_with(|recorded| recorded.borrow_mut().extend(plan.overflow_requirements));
    let actions = plan.actions;
    with_registered_buffer(|buffer| {
        // A query run while planning can emit diagnostics, which only append to the buffer, but
        // check the candidates are still where they were before rewriting any.
        let current = unresolved_names(buffer);
        if current != candidates {
            return;
        }
        let mut dropped = Vec::new();
        for ((index, span, _), action) in candidates.iter().zip(actions) {
            match action {
                Some(NameAction::Reword(name)) => {
                    let Some(BufEntry::Plain(diag)) = buffer.get_mut(*index) else {
                        continue;
                    };
                    replace_header(diag, plan_unresolved_wiring_name(&name));
                    diag.span = MultiSpan::from_span(*span);
                    if let Some(help) = unresolved_wiring_name_help(&name) {
                        diag.children.push(subdiag(Sublevel::Help, help));
                    }
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
    });
}

/// Replace the header of `diag` with the coded form of `name`, keeping the compiler's notes and
/// suggestions except the one to add a type parameter, which no wiring entry can take. The coded
/// help is added only when nothing else is left that says how to fix the name.
fn reword_header_keeping_advice(diag: &mut DiagInner, name: &UnresolvedWiringName) {
    diag.messages = vec![(
        DiagMessage::Str(plan_unresolved_wiring_name(name).into()),
        Style::NoStyle,
    )];
    retain_suggestions(
        diag,
        |suggestion| !matches!(&suggestion.msg, DiagMessage::Str(text) if is_missing_type_parameter_help_text(text)),
    );
    let has_suggestion = match &diag.suggestions {
        Suggestions::Enabled(suggestions) => !suggestions.is_empty(),
        Suggestions::Sealed(suggestions) => !suggestions.is_empty(),
        Suggestions::Disabled => false,
    };
    if !has_suggestion
        && diag.children.is_empty()
        && let Some(help) = unresolved_wiring_name_help(name)
    {
        diag.children.push(subdiag(Sublevel::Help, help));
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
